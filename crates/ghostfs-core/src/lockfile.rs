use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Lockfile (`ghost.lock`) for reproducible dependency resolution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lockfile {
    #[serde(default = "default_lockfile_version")]
    pub lockfile_version: u32,
    pub packages: BTreeMap<String, LockedPackage>,
}

fn default_lockfile_version() -> u32 {
    1
}

/// A single locked (resolved) package entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedPackage {
    pub version: String,
    pub hash: String,
    pub tarball: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub integrity: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub dependencies: BTreeMap<String, String>,
}

impl Lockfile {
    pub fn new() -> Self {
        Self {
            lockfile_version: 1,
            packages: BTreeMap::new(),
        }
    }

    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read lockfile at {}", path.display()))?;
        let lockfile: Self =
            serde_json::from_str(&content).with_context(|| "Failed to parse ghost.lock")?;
        Ok(lockfile)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content + "\n")?;
        Ok(())
    }

    pub fn lockfile_path(project_dir: &Path) -> PathBuf {
        project_dir.join("ghost.lock")
    }

    pub fn add_package(&mut self, name: &str, pkg: LockedPackage) {
        self.packages.insert(name.to_string(), pkg);
    }

    pub fn get_package(&self, name: &str) -> Option<&LockedPackage> {
        self.packages.get(name)
    }
}

impl Default for Lockfile {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_lockfile_lifecycle() {
        let mut lockfile = Lockfile::new();
        assert_eq!(lockfile.lockfile_version, 1);
        assert!(lockfile.packages.is_empty());

        let pkg = LockedPackage {
            version: "1.0.0".to_string(),
            hash: "123456".to_string(),
            tarball: "https://example.com/tarball.tgz".to_string(),
            integrity: Some("sha512-abc".to_string()),
            dependencies: BTreeMap::new(),
        };

        lockfile.add_package("my-dep", pkg);
        assert_eq!(lockfile.packages.len(), 1);

        let fetched = lockfile.get_package("my-dep").unwrap();
        assert_eq!(fetched.version, "1.0.0");
        assert_eq!(fetched.hash, "123456");

        // Test File IO
        let dir = tempdir().unwrap();
        let path = dir.path().join("ghost.lock");
        lockfile.save(&path).unwrap();
        assert!(path.exists());

        let loaded = Lockfile::load(&path).unwrap();
        assert_eq!(loaded.packages.len(), 1);
        assert_eq!(loaded.get_package("my-dep").unwrap().version, "1.0.0");
    }
}
