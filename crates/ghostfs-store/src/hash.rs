use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::Path;

use anyhow::Result;

/// Compute SHA-256 hash of raw bytes.
pub fn compute_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// Compute SHA-256 hash of a file on disk.
pub fn compute_sha256_file(path: &Path) -> Result<String> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}

/// Hash an entire directory tree deterministically (sorted entries).
pub fn hash_directory(path: &Path) -> Result<String> {
    let mut hasher = Sha256::new();
    hash_dir_recursive(path, path, &mut hasher)?;
    Ok(hex::encode(hasher.finalize()))
}

fn hash_dir_recursive(base: &Path, current: &Path, hasher: &mut Sha256) -> Result<()> {
    let mut entries: Vec<_> = std::fs::read_dir(current)?
        .filter_map(|e| e.ok())
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        let relative = path.strip_prefix(base)?;
        hasher.update(relative.to_string_lossy().as_bytes());

        if path.is_dir() {
            hash_dir_recursive(base, &path, hasher)?;
        } else {
            let mut file = std::fs::File::open(&path)?;
            let mut buffer = [0u8; 8192];
            loop {
                let n = file.read(&mut buffer)?;
                if n == 0 {
                    break;
                }
                hasher.update(&buffer[..n]);
            }
        }
    }
    Ok(())
}
