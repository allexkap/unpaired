//! Utility helpers.

use std::{
    fs,
    io::{Read, Result},
    path::Path,
};

use xxhash_rust::xxh3::Xxh3;

pub fn hash_file(path: impl AsRef<Path>) -> Result<u128> {
    let mut file = fs::File::open(path)?;
    let mut buf = [0; 1 << 20];
    let mut hasher = Xxh3::new();
    loop {
        match file.read(&mut buf)? {
            0 => return Ok(hasher.digest128()),
            n => hasher.update(&buf[..n]),
        }
    }
}

pub fn bytes_to_string(value: u64) -> String {
    const UNITS: [&str; 7] = ["", "k", "M", "G", "T", "P", "E"];

    let mut size = value as f64;
    let mut unit_idx = 0;

    while size >= 1000.0 && unit_idx < UNITS.len() - 1 {
        size /= 1000.0;
        unit_idx += 1;
    }

    let precision = if size >= 10.0 || unit_idx == 0 { 0 } else { 1 };
    format!("{size:.precision$}{}", UNITS[unit_idx])
}
