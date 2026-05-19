use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Full metadata for a package from the npm registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMetadata {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(rename = "dist-tags")]
    pub dist_tags: HashMap<String, String>,
    pub versions: HashMap<String, VersionMetadata>,
}

/// Metadata for a specific version of a package.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionMetadata {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub dependencies: Option<HashMap<String, String>>,
    #[serde(rename = "devDependencies", default)]
    pub dev_dependencies: Option<HashMap<String, String>>,
    #[serde(rename = "peerDependencies", default)]
    pub peer_dependencies: Option<HashMap<String, String>>,
    pub dist: DistInfo,
}

/// Distribution info for a package version (tarball URL, integrity hash).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistInfo {
    pub tarball: String,
    pub shasum: String,
    #[serde(default)]
    pub integrity: Option<String>,
}
