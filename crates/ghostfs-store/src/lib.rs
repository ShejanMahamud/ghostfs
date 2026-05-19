//! # ghostfs-store
//!
//! Global content-addressed package store for GhostFS.
//! Packages are stored by SHA-256 hash with SQLite metadata tracking.

mod db;
mod hash;

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

pub use db::Database;
pub use hash::{compute_sha256, compute_sha256_file, hash_directory};

/// Represents a stored package in the global store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredPackage {
    pub name: String,
    pub version: String,
    pub hash: String,
    pub size: u64,
    pub installed_at: String,
}

/// The global content-addressed package store at `~/.ghostfs/store/`.
pub struct ContentStore {
    root: PathBuf,
    db: Database,
}

impl ContentStore {
    /// Create a new store at the default location (`~/.ghostfs/store/`).
    pub fn new() -> Result<Self> {
        let root = Self::default_store_path()?;
        Self::with_path(root)
    }

    /// Create a store at a custom path.
    pub fn with_path(root: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&root)?;
        let db_path = root.join("ghostfs.db");
        let db = Database::open(&db_path)?;
        Ok(Self { root, db })
    }

    /// Return the default store path: `~/.ghostfs/store/`.
    pub fn default_store_path() -> Result<PathBuf> {
        let home = dirs::home_dir().context("Could not determine home directory")?;
        Ok(home.join(".ghostfs").join("store"))
    }

    /// Store a package from an extracted directory into the content-addressed store.
    pub fn store_package(
        &self,
        name: &str,
        version: &str,
        source_dir: &Path,
    ) -> Result<StoredPackage> {
        let hash = hash_directory(source_dir)?;
        let target = self.package_path(&hash);

        if !target.exists() {
            copy_dir_recursive(source_dir, &target)?;
        }

        let size = dir_size(&target)?;
        let installed_at = chrono::Utc::now().to_rfc3339();

        let pkg = StoredPackage {
            name: name.to_string(),
            version: version.to_string(),
            hash,
            size,
            installed_at,
        };

        self.db.insert_package(&pkg)?;
        Ok(pkg)
    }

    /// Get the filesystem path to a stored package by its content hash.
    pub fn package_path(&self, hash: &str) -> PathBuf {
        // Use first 2 hex chars as shard directory for better FS distribution
        let shard = &hash[..2.min(hash.len())];
        self.root.join(shard).join(hash)
    }

    /// Check if a package already exists in the store.
    pub fn has_package(&self, name: &str, version: &str) -> Result<bool> {
        self.db.has_package(name, version)
    }

    /// Get stored package metadata.
    pub fn get_package(&self, name: &str, version: &str) -> Result<Option<StoredPackage>> {
        self.db.get_package(name, version)
    }

    /// List all stored packages.
    pub fn list_packages(&self) -> Result<Vec<StoredPackage>> {
        self.db.list_packages()
    }

    /// Get the root path of the store.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Get a reference to the metadata database.
    pub fn db(&self) -> &Database {
        &self.db
    }
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dst_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&entry.path(), &dst_path)?;
        } else {
            std::fs::copy(entry.path(), &dst_path)?;
        }
    }
    Ok(())
}

fn dir_size(path: &Path) -> Result<u64> {
    let mut size = 0u64;
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                size += dir_size(&entry.path())?;
            } else {
                size += entry.metadata()?.len();
            }
        }
    }
    Ok(size)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_content_store_lifecycle() {
        let store_dir = tempdir().unwrap();
        let store = ContentStore::with_path(store_dir.path().to_path_buf()).unwrap();

        // Create a fake package directory to store
        let pkg_src = tempdir().unwrap();
        let file_path = pkg_src.path().join("index.js");
        std::fs::write(&file_path, "console.log('hello');").unwrap();

        // Store it
        let stored = store.store_package("my-lib", "1.2.3", pkg_src.path()).unwrap();
        assert_eq!(stored.name, "my-lib");
        assert_eq!(stored.version, "1.2.3");
        assert!(stored.size > 0);

        // Verify package path and package exists
        let dest_path = store.package_path(&stored.hash);
        assert!(dest_path.exists());
        assert!(dest_path.join("index.js").exists());
        
        assert!(store.has_package("my-lib", "1.2.3").unwrap());
        
        let retrieved = store.get_package("my-lib", "1.2.3").unwrap().unwrap();
        assert_eq!(retrieved.hash, stored.hash);
    }
}
