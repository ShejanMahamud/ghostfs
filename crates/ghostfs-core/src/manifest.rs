use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Project manifest (`ghost.json`) — equivalent to `package.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub name: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub dependencies: BTreeMap<String, String>,
    #[serde(
        rename = "devDependencies",
        default,
        skip_serializing_if = "BTreeMap::is_empty"
    )]
    pub dev_dependencies: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub scripts: BTreeMap<String, String>,
}

fn default_version() -> String {
    "0.1.0".to_string()
}

impl Manifest {
    /// Create a new empty manifest with the given project name.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            version: "0.1.0".to_string(),
            description: None,
            dependencies: BTreeMap::new(),
            dev_dependencies: BTreeMap::new(),
            scripts: BTreeMap::new(),
        }
    }

    /// Load a manifest from a file path.
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read manifest at {}", path.display()))?;
        let manifest: Self =
            serde_json::from_str(&content).with_context(|| "Failed to parse manifest JSON")?;
        Ok(manifest)
    }

    /// Save the manifest to a file path (pretty-printed JSON).
    pub fn save(&self, path: &Path) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content + "\n")?;
        Ok(())
    }

    /// Get the canonical manifest path for a project directory.
    pub fn manifest_path(project_dir: &Path) -> PathBuf {
        project_dir.join("ghost.json")
    }

    /// Try to load from `ghost.json`, falling back to `package.json`.
    pub fn load_from_dir(project_dir: &Path) -> Result<Self> {
        let ghost_path = project_dir.join("ghost.json");
        if ghost_path.exists() {
            return Self::load(&ghost_path);
        }
        let pkg_path = project_dir.join("package.json");
        if pkg_path.exists() {
            return Self::load(&pkg_path);
        }
        anyhow::bail!(
            "No ghost.json or package.json found in {}",
            project_dir.display()
        )
    }

    /// Add a dependency to the manifest.
    pub fn add_dependency(&mut self, name: &str, version: &str) {
        self.dependencies
            .insert(name.to_string(), version.to_string());
    }

    /// Remove a dependency (from either deps or devDeps). Returns true if found.
    pub fn remove_dependency(&mut self, name: &str) -> bool {
        self.dependencies.remove(name).is_some() || self.dev_dependencies.remove(name).is_some()
     }
 }

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_manifest_manipulation() {
        let mut manifest = Manifest::new("my-test-app");
        assert_eq!(manifest.name, "my-test-app");
        assert_eq!(manifest.version, "0.1.0");
        assert!(manifest.dependencies.is_empty());

        // Add dependencies
        manifest.add_dependency("react", "^19.0.0");
        assert_eq!(manifest.dependencies.get("react").unwrap(), "^19.0.0");

        // Remove dependency
        assert!(manifest.remove_dependency("react"));
        assert!(!manifest.remove_dependency("react")); // Already removed
    }

    #[test]
    fn test_manifest_file_io() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("ghost.json");

        let mut manifest = Manifest::new("io-app");
        manifest.add_dependency("lodash", "^4.17.21");
        manifest.save(&path).unwrap();

        assert!(path.exists());

        let loaded = Manifest::load(&path).unwrap();
        assert_eq!(loaded.name, "io-app");
        assert_eq!(loaded.dependencies.get("lodash").unwrap(), "^4.17.21");

        let loaded_from_dir = Manifest::load_from_dir(dir.path()).unwrap();
        assert_eq!(loaded_from_dir.name, "io-app");
    }
}
