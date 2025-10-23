use std::{
    cmp::Reverse,
    collections::HashMap,
    fmt::{self, Display},
    fs::{DirEntry, File},
    hash::Hasher,
    io::{self, BufReader, Read},
    path::{self, Path, PathBuf},
};

use seahash::SeaHasher;
use walkdir::WalkDir;

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
struct FileInfo {
    path: PathBuf,
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Hash, Clone, Copy, Debug)]
struct FileData {
    size: u64,
    hash: Option<u64>,
}

impl Display for FileData {
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

        write!(f, "File({hash_str}: {size:.2}{unit})")
    }
}

impl FileData {
    fn new<T: AsRef<Path>>(path: &T) -> io::Result<FileData> {
        let path = path.as_ref();
        Ok(FileData {
            size: path.metadata()?.len(),
            hash: Some(hash_file(path)?),
        })
    }
}

fn hash_file<T: AsRef<Path>>(path: T) -> io::Result<u64> {
    let mut file = File::open(path)?;
    let mut buf = [0; 4096];
    let mut hasher = SeaHasher::new();
    loop {
        match file.read(&mut buf)? {
            0 => return Ok(hasher.finish()),
            _ => hasher.write(&buf),
        }
    }
}

fn main() {
    let root = path::absolute(".").unwrap();
    let mut files: HashMap<FileData, Vec<FileInfo>> = HashMap::new();
    for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }

        let data = match FileData::new(&entry.path()) {
            Ok(data) => data,
            Err(err) => {
                println!("{err:?}");
                continue;
            }
        };

        let info = FileInfo {
            path: entry.path().to_owned(),
        };

        if let Some(v) = files.get_mut(&data) {
            v.push(info)
        } else {
            files.insert(data, vec![info]);
        }
    }

    let mut sorted_files: Vec<(FileData, Vec<FileInfo>)> = files.drain().collect();
    sorted_files.sort_by_key(|k| Reverse((k.0.size * k.1.len() as u64, k.0.hash)));

    for (data, infos) in sorted_files.into_iter() {
        println!("\n{data}");
        for info in infos {
            println!("{info:?}");
        }
    }
}
