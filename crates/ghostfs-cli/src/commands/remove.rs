use anyhow::Result;
use console::style;
use ghostfs_core::Manifest;
use std::path::Path;

/// Remove a package from ghost.json dependencies.
pub fn run(project_dir: &Path, package: &str) -> Result<()> {
    let manifest_path = Manifest::manifest_path(project_dir);
    let mut manifest = Manifest::load_from_dir(project_dir)?;

    if manifest.remove_dependency(package) {
        manifest.save(&manifest_path)?;
        println!(
            "{} Removed {}",
            style("✓").green().bold(),
            style(package).cyan()
        );
        println!("  Run {} to update.", style("ghost install").dim());
    } else {
        println!(
            "{} Package '{}' not found in dependencies",
            style("⚠").yellow(),
            package
        );
    }

    Ok(())
}
