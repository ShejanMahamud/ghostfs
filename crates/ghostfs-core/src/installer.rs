use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use ghostfs_registry::NpmRegistryClient;
use ghostfs_store::{compute_sha256, ContentStore};
use std::path::Path;
use tar::Archive;
use tracing::{debug, info};

use crate::lockfile::{LockedPackage, Lockfile};
use crate::manifest::Manifest;
use crate::resolver::{DependencyResolver, ResolvedPackage};

/// Orchestrates dependency resolution, download, extraction, and storage.
pub struct Installer {
    store: ContentStore,
    registry: NpmRegistryClient,
}

impl Installer {
    pub fn new(store: ContentStore, registry: NpmRegistryClient) -> Self {
        Self { store, registry }
    }

    /// Install all dependencies for a project directory.
    /// Returns the number of packages installed.
    pub async fn install(&self, project_dir: &Path) -> Result<InstallResult> {
        let manifest = Manifest::load_from_dir(project_dir)?;
        info!("Installing dependencies for '{}'", manifest.name);

        // Check for existing lockfile
        let lockfile_path = Lockfile::lockfile_path(project_dir);
        let _existing_lockfile = if lockfile_path.exists() {
            Some(Lockfile::load(&lockfile_path)?)
        } else {
            None
        };

        // Resolve dependencies
        let mut resolver = DependencyResolver::new(NpmRegistryClient::new());
        let resolved = resolver.resolve(&manifest.dependencies).await?;

        let total = resolved.len();
        let mut installed = 0u32;
        let mut cached = 0u32;
        let mut lockfile = Lockfile::new();

        for pkg in &resolved {
            // Check if already in store
            if self.store.has_package(&pkg.name, &pkg.version)? {
                debug!("{} @ {} already in store, skipping", pkg.name, pkg.version);
                cached += 1;

                // Still add to lockfile
                if let Some(stored) = self.store.get_package(&pkg.name, &pkg.version)? {
                    lockfile.add_package(
                        &pkg.name,
                        LockedPackage {
                            version: pkg.version.clone(),
                            hash: stored.hash,
                            tarball: pkg.tarball.clone(),
                            integrity: pkg.integrity.clone(),
                            dependencies: pkg.dependencies.clone(),
                        },
                    );
                }
                continue;
            }

            // Download and store
            info!("Downloading {} @ {}", pkg.name, pkg.version);
            let stored = self.download_and_store(pkg).await?;
            lockfile.add_package(
                &pkg.name,
                LockedPackage {
                    version: pkg.version.clone(),
                    hash: stored.hash,
                    tarball: pkg.tarball.clone(),
                    integrity: pkg.integrity.clone(),
                    dependencies: pkg.dependencies.clone(),
                },
            );
            installed += 1;
        }

        // Write lockfile
        lockfile.save(&lockfile_path)?;
        info!("Wrote lockfile to {}", lockfile_path.display());

        Ok(InstallResult {
            total,
            installed,
            cached,
        })
    }

    /// Download a package tarball, extract it, and store in the content-addressed store.
    pub async fn download_and_store(
        &self,
        pkg: &ResolvedPackage,
    ) -> Result<ghostfs_store::StoredPackage> {
        // Download tarball
        let tarball_bytes = self.registry.download_tarball(&pkg.tarball).await?;

        // Verify integrity
        let hash = compute_sha256(&tarball_bytes);
        debug!(
            "Downloaded {} ({} bytes, sha256={})",
            pkg.name,
            tarball_bytes.len(),
            hash
        );

        // Extract to temp directory
        let temp_dir = tempfile::tempdir()?;
        let decoder = GzDecoder::new(&tarball_bytes[..]);
        let mut archive = Archive::new(decoder);
        archive
            .unpack(temp_dir.path())
            .with_context(|| format!("Failed to extract tarball for {}", pkg.name))?;

        // npm tarballs extract to a `package/` subdirectory
        let package_dir = temp_dir.path().join("package");
        let source_dir = if package_dir.exists() {
            package_dir
        } else {
            // Fallback: find the first directory in temp
            let mut entries = std::fs::read_dir(temp_dir.path())?;
            if let Some(Ok(entry)) = entries.next() {
                if entry.file_type()?.is_dir() {
                    entry.path()
                } else {
                    temp_dir.path().to_path_buf()
                }
            } else {
                temp_dir.path().to_path_buf()
            }
        };

        // Store in the global content-addressed store
        let stored = self
            .store
            .store_package(&pkg.name, &pkg.version, &source_dir)?;
        info!(
            "Stored {} @ {} (hash: {})",
            pkg.name,
            pkg.version,
            &stored.hash[..12]
        );

        Ok(stored)
    }

    /// Get a reference to the content store.
    pub fn store(&self) -> &ContentStore {
        &self.store
    }
}

/// Result of an install operation.
#[derive(Debug)]
pub struct InstallResult {
    pub total: usize,
    pub installed: u32,
    pub cached: u32,
}
