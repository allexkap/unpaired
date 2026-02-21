use std::{
    cmp::Reverse,
    collections::{HashMap, hash_map},
    fmt,
    fs::{self},
    hash::Hasher,
    io::{self, Read},
    path::{Path, PathBuf},
    time::SystemTime,
};

use seahash::SeaHasher;
use serde::{Deserialize, Serialize};
use tqdm::Iter;
use walkdir::WalkDir;

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub struct MetadataSnapshot {
    len: u64,
    accessed: Option<SystemTime>,
    modified: Option<SystemTime>,
    created: Option<SystemTime>,
    ino: Option<u64>,
}

fn get_ino(value: &fs::Metadata) -> Option<u64> {
    #[cfg(target_os = "linux")]
    return Some(std::os::unix::fs::MetadataExt::ino(value));

    #[cfg(not(target_os = "linux"))]
    return None;
}

impl From<fs::Metadata> for MetadataSnapshot {
    fn from(value: fs::Metadata) -> Self {
        Self {
            len: value.len(),
            accessed: value.accessed().ok(),
            modified: value.modified().ok(),
            created: value.created().ok(),
            ino: get_ino(&value),
        }
    }
}

/// A representation of a file in the filesystem.
///
/// It does **not** store the file contents.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct FileInfo {
    /// Full path to the file.
    path: PathBuf,

    /// File metadata
    meta: MetadataSnapshot,
}

impl FileInfo {
    fn new(path: impl AsRef<Path>, meta: fs::Metadata) -> FileInfo {
        FileInfo {
            path: path.as_ref().to_owned(),
            meta: meta.into(),
        }
    }
}

impl fmt::Display for FileInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path.display())
    }
}

/// File identification data with LoD.
///
/// Used for comparing and grouping files by their actual content.
#[derive(Clone, Copy, Serialize, Deserialize, PartialOrd, Ord, PartialEq, Eq, Hash, Debug)]
pub struct FileData {
    /// File size in bytes.
    size: u64,

    /// Hash of file contents.
    hash: Option<u64>,
}

impl FileData {
    fn new(info: &FileInfo) -> io::Result<FileData> {
        Ok(FileData {
            size: info.meta.len,
            hash: None,
        })
    }

    fn with_hash(&self, info: &FileInfo) -> io::Result<FileData> {
        Ok(FileData {
            size: self.size,
            hash: Some(hash_file(&info.path)?),
        })
    }
}

impl fmt::Display for FileData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const UNITS: [&str; 5] = ["B", "kB", "MB", "GB", "TB"];
        let mut unit = UNITS[4];
        let mut size = self.size as f64;

        for p in UNITS {
            if size <= 1000.0 {
                unit = p;
                break;
            }
            size /= 1000.0;
        }

        let hash_str = match self.hash {
            Some(hash) => format!("{hash:016x}"),
            None => "-".to_owned(),
        };

        write!(
            f,
            "FileData({hash_str}: {size:.0$}{unit})",
            if unit == UNITS[0] { 0 } else { 1 }
        )
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
enum FileGroup {
    Uniq(FileInfo),
    Many(Vec<FileInfo>),
}

#[derive(Default, Clone, Serialize, Deserialize, Debug)]
pub struct FileIndex {
    files: HashMap<FileData, FileGroup>,
}

impl FileIndex {
    pub fn fast_add(&mut self, info: FileInfo, data: FileData) {
        match self.files.entry(data) {
            hash_map::Entry::Occupied(mut entry) => {
                let prev_info = entry.get_mut();
                match prev_info {
                    FileGroup::Uniq(file_info) => {
                        *prev_info = FileGroup::Many(vec![file_info.clone(), info]);
                    }
                    FileGroup::Many(file_group) => file_group.push(info),
                }
            }
            hash_map::Entry::Vacant(entry) => {
                entry.insert(FileGroup::Uniq(info));
            }
        }
    }

    pub fn len(&self) -> usize {
        self.files.len()
    }

    pub fn remove_ambiguous(&mut self) -> Vec<(FileInfo, FileData)> {
        self.files
            .iter_mut()
            .filter_map(|entry| match entry.1 {
                FileGroup::Uniq(_) => None,
                FileGroup::Many(file_group) => {
                    Some(file_group.drain(..).zip(std::iter::repeat(*entry.0)))
                }
            })
            .flatten()
            .collect()
    }

    pub fn get_preview(&self) -> Vec<(&FileData, &Vec<FileInfo>)> {
        let mut sorted_files: Vec<_> = self
            .files
            .iter()
            .filter_map(|entry| match entry {
                (data, FileGroup::Many(file_group)) if file_group.len() > 1 => {
                    Some((data, file_group))
                }
                _ => None,
            })
            .collect();
        sorted_files.sort_by_key(|k| Reverse((k.0.size * k.1.len() as u64, k.0.hash)));
        sorted_files
    }

    pub fn dump(&self, path: impl AsRef<Path>) -> io::Result<()> {
        let file = fs::File::create(path)?;
        let writer = io::BufWriter::new(file);
        postcard::to_io(&self, writer).expect("failed to dump file index");
        Ok(())
    }
}

fn hash_file(path: impl AsRef<Path>) -> io::Result<u64> {
    let mut file = fs::File::open(path)?;
    let mut buf = [0; 4096];
    let mut hasher = SeaHasher::new();
    loop {
        match file.read(&mut buf)? {
            0 => return Ok(hasher.finish()),
            n => hasher.write(&buf[..n]),
        }
    }
}

fn process_entry(entry: &walkdir::DirEntry) -> io::Result<(FileInfo, FileData)> {
    let info = FileInfo::new(entry.path(), entry.metadata()?);
    let data = FileData::new(&info)?;
    return Ok((info, data));
}

fn skip_file(path: impl AsRef<Path>, err: std::io::Error) {
    println!("{}: {}", path.as_ref().display(), err);
}

pub fn process_dir(path: impl AsRef<Path>) -> FileIndex {
    let mut file_index = FileIndex::default();

    for entry in WalkDir::new(path).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }

        match process_entry(&entry) {
            Ok((info, data)) => file_index.fast_add(info, data),
            Err(err) => skip_file(entry.path(), err),
        }
    }

    let ambiguous_files = file_index.remove_ambiguous();

    for (info, data) in ambiguous_files.into_iter().tqdm() {
        match data.with_hash(&info) {
            Ok(data) => file_index.fast_add(info, data),
            Err(err) => skip_file(info.path, err),
        };
    }

    file_index
}
