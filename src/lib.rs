use std::{io, path::Path};

use tqdm::Iter;
use walkdir::WalkDir;

use crate::file_index::{FileData, FileIndex, FileInfo};

mod file_index;
mod utils;

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
