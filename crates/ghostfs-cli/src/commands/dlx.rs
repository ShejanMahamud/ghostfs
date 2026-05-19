use anyhow::{Context, Result};
use console::style;
use ghostfs_core::{
    installer::Installer,
    lockfile::{LockedPackage, Lockfile},
    resolver::DependencyResolver,
};
use ghostfs_registry::NpmRegistryClient;
use ghostfs_store::ContentStore;
use std::collections::BTreeMap;

/// Run a package executable from the registry dynamically without installing it locally.
pub async fn run(specifier: &str, args: &[String]) -> Result<()> {
    // Parse specifier: name@version
    let (name, version_req) = if let Some(stripped) = specifier.strip_prefix('@') {
        if let Some((n, v)) = stripped.rsplit_once('@') {
            (format!("@{}", n), v.to_string())
        } else {
            (specifier.to_string(), "latest".to_string())
        }
    } else if let Some((n, v)) = specifier.rsplit_once('@') {
        (n.to_string(), v.to_string())
    } else {
        (specifier.to_string(), "latest".to_string())
    };

    println!(
        "{} Running {} dynamically...",
        style("◌").blue(),
        style(&specifier).cyan()
    );

    let store = ContentStore::new()?;
    let registry = NpmRegistryClient::new();
    let installer = Installer::new(ContentStore::new()?, NpmRegistryClient::new());

    // Resolve dependencies
    let mut resolver = DependencyResolver::new(registry);
    let mut deps = BTreeMap::new();
    deps.insert(name.clone(), version_req.clone());
    let resolved = resolver.resolve(&deps).await?;

    // Download/fetch to store and build lockfile
    let mut lockfile = Lockfile::new();
    let mut target_pkg_resolved = None;

    for pkg in &resolved {
        let stored = if store.has_package(&pkg.name, &pkg.version)? {
            store.get_package(&pkg.name, &pkg.version)?.unwrap()
        } else {
            installer.download_and_store(pkg).await?
        };

        lockfile.add_package(
            &pkg.name,
            LockedPackage {
                version: pkg.version.clone(),
                hash: stored.hash.clone(),
                tarball: pkg.tarball.clone(),
                integrity: pkg.integrity.clone(),
                dependencies: pkg.dependencies.clone(),
            },
        );

        if pkg.name == name {
            target_pkg_resolved = Some((pkg.clone(), stored));
        }
    }

    let (_target_pkg, stored_pkg) =
        target_pkg_resolved.context("Failed to find resolved target package in dependency tree")?;

    // Read package.json to find binary executable path
    let pkg_dir = store.package_path(&stored_pkg.hash);
    let pkg_json_path = pkg_dir.join("package.json");
    let pkg_json_content = std::fs::read_to_string(&pkg_json_path)
        .with_context(|| format!("Failed to read package.json at {}", pkg_json_path.display()))?;
    let pkg_json: serde_json::Value = serde_json::from_str(&pkg_json_content)?;

    let bin_subpath = match pkg_json.get("bin") {
        Some(serde_json::Value::String(s)) => Some(s.clone()),
        Some(serde_json::Value::Object(map)) => {
            let simple_name = name.split('/').next_back().unwrap_or(&name);
            if let Some(serde_json::Value::String(s)) = map.get(simple_name) {
                Some(s.clone())
            } else if let Some(serde_json::Value::String(s)) = map.values().next() {
                Some(s.clone())
            } else {
                None
            }
        }
        _ => None,
    };

    let bin_subpath = bin_subpath.with_context(|| {
        format!(
            "Package '{}' does not define any executable binary in its package.json",
            name
        )
    })?;

    let bin_js_path = pkg_dir.join(bin_subpath);

    // Create temporary lockfile
    let temp_lockfile = tempfile::Builder::new()
        .prefix("ghost-dlx-")
        .suffix(".lock")
        .tempfile()?;
    lockfile.save(temp_lockfile.path())?;

    // Determine path to global resolver.js
    let home = dirs::home_dir().context("Could not determine home directory")?;
    let resolver_path = home.join(".ghostfs").join("runtime").join("resolver.js");

    if !resolver_path.exists() {
        // Auto-install hooks for smooth user experience
        super::hooks::run()?;
    }

    let resolver_str = resolver_path.to_string_lossy().replace('\\', "/");

    // Spawn node process executing the binary virtually
    let mut cmd = std::process::Command::new("node");
    cmd.env("GHOST_LOCKFILE_PATH", temp_lockfile.path());
    cmd.env("NODE_OPTIONS", format!("--require \"{}\"", resolver_str));
    cmd.arg(&bin_js_path);
    cmd.args(args);

    let mut child = cmd.spawn().context("Failed to spawn Node.js process")?;
    let status = child.wait().context("Failed to wait for Node.js process")?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}
