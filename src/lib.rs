use std::{
    cmp::Reverse,
    collections::HashMap,
    fmt,
    hash::Hasher,
    io::{self, Read},
    path::{Path, PathBuf},
};

use seahash::SeaHasher;
use walkdir::WalkDir;

mod fiemap;

#[derive(Clone, Debug)]
pub struct FileInfo {
    pub path: PathBuf,
    pub meta: std::fs::Metadata,
    pub fe_physical: Option<u64>,
}

impl FileInfo {
    fn new<T: AsRef<Path>>(
        path: &T,
        meta: std::fs::Metadata,
        fe_physical: Option<u64>,
    ) -> FileInfo {
        FileInfo {
            path: path.as_ref().to_owned(),
            meta,
            fe_physical,
        }
    }
}

impl fmt::Display for FileInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path.display())
    }
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct FileData {
    pub size: u64,
    pub hash: Option<u64>,
}

impl FileData {
    fn new(info: &FileInfo) -> io::Result<FileData> {
        Ok(FileData {
            size: info.meta.len(),
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

#[derive(Default)]
pub struct Files {
    inner: HashMap<FileData, Vec<FileInfo>>,
}

impl Files {
    pub fn add(&mut self, info: FileInfo, data: FileData) {
        let Some(same_files) = self.inner.get_mut(&data) else {
            self.inner.insert(data, vec![info]);
            return;
        };

        if data.hash == None {
            if same_files.len() > 0 {
                for inner_info in std::mem::take(same_files) {
                    let inner_data = data.with_hash(&inner_info).unwrap();
                    self.add(inner_info, inner_data);
                }
            }
            let data = data.with_hash(&info).unwrap();
            self.add(info, data);
        } else {
            same_files.push(info);
        }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn get_preview(&self) -> Vec<(&FileData, &Vec<FileInfo>)> {
        let mut sorted_files: Vec<_> = self.inner.iter().filter(|k| k.1.len() > 0).collect();
        sorted_files.sort_by_key(|k| Reverse((k.0.size * k.1.len() as u64, k.0.hash)));
        sorted_files
    }
}

fn hash_file<T: AsRef<Path>>(path: &T) -> io::Result<u64> {
    let mut file = std::fs::File::open(path)?;
    let mut buf = [0; 4096];
    let mut hasher = SeaHasher::new();
    loop {
        match file.read(&mut buf)? {
            0 => return Ok(hasher.finish()),
            n => hasher.write(&buf[..n]),
        }
    }
}

pub fn process_entry(entry: &walkdir::DirEntry) -> io::Result<(FileInfo, FileData)> {
    let fe_physical = fiemap::read_fiemap(entry.path(), Some(1))?
        .1
        .get(0)
        .map(|f| f.fe_physical);

    let info = FileInfo::new(&entry.path(), entry.metadata()?, fe_physical);
    let data = FileData::new(&info)?;
    return Ok((info, data));
}

pub fn process_dir<T: AsRef<Path>>(path: &T) -> Files {
    let mut files = Files::default();
    for entry in WalkDir::new(path).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }

        match process_entry(&entry) {
            Ok((info, data)) => files.add(info, data),
            Err(err) => println!("{}: {}", entry.path().display(), err),
        }
    }
    files
}
