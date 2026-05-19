use anyhow::{Context, Result};
use std::path::Path;
use tracing::{debug, info};

use crate::lockfile::Lockfile;
use ghostfs_store::ContentStore;

/// Links packages from the global store into a project's `node_modules/` using
/// directory junctions (Windows) or symlinks (Unix). This provides 100%
/// compatibility with existing Node.js tooling while still deduplicating on disk.
pub struct Linker {
    store: ContentStore,
}

impl Linker {
    pub fn new(store: ContentStore) -> Self {
        Self { store }
    }

    /// Create `node_modules/` in the project directory with symlinks to the
    /// global store, based on the lockfile.
    pub fn link(&self, project_dir: &Path) -> Result<LinkResult> {
        let lockfile_path = Lockfile::lockfile_path(project_dir);
        if !lockfile_path.exists() {
            anyhow::bail!("No ghost.lock found. Run 'ghost install' first.");
        }

        let lockfile = Lockfile::load(&lockfile_path)?;
        let node_modules = project_dir.join("node_modules");

        // Clean existing node_modules
        if node_modules.exists() {
            std::fs::remove_dir_all(&node_modules)
                .with_context(|| "Failed to clean existing node_modules")?;
        }
        std::fs::create_dir_all(&node_modules)?;

        // Write .ghostfs marker so tools know this is managed
        std::fs::write(
            node_modules.join(".ghostfs"),
            "This node_modules is managed by GhostFS.\nDo not modify manually.\n",
        )?;

        let mut linked = 0u32;
        let mut errors = Vec::new();

        for (name, locked_pkg) in &lockfile.packages {
            let store_path = self.store.package_path(&locked_pkg.hash);

            if !store_path.exists() {
                errors.push(format!(
                    "{} @ {} not found in store (hash: {})",
                    name,
                    locked_pkg.version,
                    &locked_pkg.hash[..12]
                ));
                continue;
            }

            let link_target = if name.starts_with('@') {
                // Scoped package: node_modules/@scope/pkg
                let parts: Vec<&str> = name.splitn(2, '/').collect();
                if parts.len() == 2 {
                    let scope_dir = node_modules.join(parts[0]);
                    std::fs::create_dir_all(&scope_dir)?;
                    scope_dir.join(parts[1])
                } else {
                    node_modules.join(name)
                }
            } else {
                node_modules.join(name)
            };

            create_link(&link_target, &store_path)
                .with_context(|| format!("Failed to link {}", name))?;

            debug!(
                "Linked {} -> {}",
                link_target.display(),
                store_path.display()
            );
            linked += 1;
        }

        // Also create .package-lock.json for Node.js compat
        create_package_lock(&node_modules, &lockfile)?;

        info!("Linked {} packages into node_modules/", linked);

        Ok(LinkResult { linked, errors })
    }

    /// Remove the managed node_modules directory.
    pub fn unlink(&self, project_dir: &Path) -> Result<()> {
        let node_modules = project_dir.join("node_modules");
        if node_modules.exists() {
            let marker = node_modules.join(".ghostfs");
            if marker.exists() {
                std::fs::remove_dir_all(&node_modules)?;
                info!("Removed node_modules/");
            } else {
                anyhow::bail!(
                    "node_modules/ exists but is not managed by GhostFS. Remove it manually."
                );
            }
        }
        Ok(())
    }
}

/// Create a symlink (Unix) or directory junction (Windows).
fn create_link(link: &Path, target: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target, link)?;
    }

    #[cfg(windows)]
    {
        // Use junction (no admin privileges needed, unlike symlinks on Windows)
        junction::create(target, link).with_context(|| {
            format!(
                "Failed to create junction: {} -> {}",
                link.display(),
                target.display()
            )
        })?;
    }

    Ok(())
}

/// Create a minimal .package-lock.json so Node.js recognizes the structure.
fn create_package_lock(node_modules: &Path, lockfile: &Lockfile) -> Result<()> {
    let mut packages = serde_json::Map::new();

    for (name, locked_pkg) in &lockfile.packages {
        let mut entry = serde_json::Map::new();
        entry.insert(
            "version".into(),
            serde_json::Value::String(locked_pkg.version.clone()),
        );
        entry.insert(
            "resolved".into(),
            serde_json::Value::String(locked_pkg.tarball.clone()),
        );
        if let Some(integrity) = &locked_pkg.integrity {
            entry.insert(
                "integrity".into(),
                serde_json::Value::String(integrity.clone()),
            );
        }
        packages.insert(
            format!("node_modules/{}", name),
            serde_json::Value::Object(entry),
        );
    }

    let lock = serde_json::json!({
        "name": "ghostfs-managed",
        "lockfileVersion": 3,
        "packages": packages,
    });

    let content = serde_json::to_string_pretty(&lock)?;
    std::fs::write(node_modules.join(".package-lock.json"), content + "\n")?;
    Ok(())
}

/// Result of a link operation.
#[derive(Debug)]
pub struct LinkResult {
    pub linked: u32,
    pub errors: Vec<String>,
}
