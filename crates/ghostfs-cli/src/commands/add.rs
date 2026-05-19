use anyhow::Result;
use console::style;
use ghostfs_core::Manifest;
use ghostfs_registry::NpmRegistryClient;
use std::path::Path;

/// Add a package to ghost.json dependencies.
pub async fn run(project_dir: &Path, package: &str, dev: bool) -> Result<()> {
    let manifest_path = Manifest::manifest_path(project_dir);
    let mut manifest = Manifest::load_from_dir(project_dir)?;

    // Parse package name and optional version
    let (name, version_req) = if let Some(idx) = package.rfind('@') {
        if idx == 0 {
            // Scoped package without version, e.g. @scope/pkg
            (package.to_string(), "latest".to_string())
        } else {
            (package[..idx].to_string(), package[idx + 1..].to_string())
        }
    } else {
        (package.to_string(), "latest".to_string())
    };

    // Resolve "latest" to actual version
    let resolved_version = if version_req == "latest" {
        println!(
            "{} Resolving {}...",
            style("◌").blue(),
            style(&name).cyan()
        );
        let registry = NpmRegistryClient::new();
        let metadata = registry.get_package_metadata(&name).await?;
        let latest = metadata
            .dist_tags
            .get("latest")
            .cloned()
            .unwrap_or_else(|| "0.0.0".to_string());
        format!("^{}", latest)
    } else {
        version_req
    };

    if dev {
        manifest
            .dev_dependencies
            .insert(name.clone(), resolved_version.clone());
    } else {
        manifest
            .dependencies
            .insert(name.clone(), resolved_version.clone());
    }

    manifest.save(&manifest_path)?;

    let dep_type = if dev { "devDependency" } else { "dependency" };
    println!(
        "{} Added {} {} as {}",
        style("✓").green().bold(),
        style(&name).cyan(),
        style(&resolved_version).dim(),
        dep_type
    );

    Ok(())
}
