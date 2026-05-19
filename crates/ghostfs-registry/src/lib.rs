//! # ghostfs-registry
//!
//! Async client for the npm package registry.
//! Fetches package metadata and downloads tarballs.

mod types;

use anyhow::{Context, Result};
use reqwest::Client;

pub use types::*;

/// Client for interacting with the npm registry.
pub struct NpmRegistryClient {
    client: Client,
    registry_url: String,
}

impl NpmRegistryClient {
    /// Create a client pointing to the default npm registry.
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            registry_url: "https://registry.npmjs.org".to_string(),
        }
    }

    /// Create a client pointing to a custom registry URL.
    pub fn with_registry(registry_url: String) -> Self {
        Self {
            client: Client::new(),
            registry_url,
        }
    }

    /// Fetch full package metadata (all versions) from the registry.
    pub async fn get_package_metadata(&self, name: &str) -> Result<PackageMetadata> {
        let url = format!("{}/{}", self.registry_url, encode_package_name(name));
        let response = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await
            .with_context(|| format!("Failed to fetch metadata for '{}'", name))?;

        if !response.status().is_success() {
            anyhow::bail!(
                "Registry returned {} for package '{}'",
                response.status(),
                name
            );
        }

        let metadata: PackageMetadata = response
            .json()
            .await
            .with_context(|| format!("Failed to parse metadata for '{}'", name))?;

        Ok(metadata)
    }

    /// Download a package tarball and return the raw bytes.
    pub async fn download_tarball(&self, tarball_url: &str) -> Result<Vec<u8>> {
        let response = self
            .client
            .get(tarball_url)
            .send()
            .await
            .with_context(|| format!("Failed to download tarball from {}", tarball_url))?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to download tarball: HTTP {}", response.status());
        }

        let bytes = response.bytes().await?;
        Ok(bytes.to_vec())
    }
}

impl Default for NpmRegistryClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Encode scoped package names (e.g. `@scope/pkg` → `@scope%2fpkg`).
fn encode_package_name(name: &str) -> String {
    if name.starts_with('@') {
        name.replace('/', "%2f")
    } else {
        name.to_string()
    }
}
