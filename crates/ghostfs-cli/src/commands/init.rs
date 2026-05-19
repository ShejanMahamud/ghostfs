use anyhow::Result;
use console::style;
use ghostfs_core::Manifest;
use std::path::Path;

/// Initialize a new GhostFS project in the given directory.
pub fn run(project_dir: &Path) -> Result<()> {
    let manifest_path = Manifest::manifest_path(project_dir);

    if manifest_path.exists() {
        println!(
            "{} ghost.json already exists in {}",
            style("⚠").yellow(),
            project_dir.display()
        );
        return Ok(());
    }

    // Derive project name from directory
    let name = project_dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "my-project".to_string());

    let mut manifest = Manifest::new(&name);
    manifest.description = Some("A GhostFS project".to_string());
    manifest
        .scripts
        .insert("dev".to_string(), "node index.js".to_string());

    manifest.save(&manifest_path)?;

    println!(
        "{} Initialized GhostFS project '{}'",
        style("✓").green().bold(),
        style(&name).cyan()
    );
    println!(
        "  {} ghost.json",
        style("created").dim()
    );
    println!();
    println!("  Next steps:");
    println!(
        "    {} ghost add react",
        style("$").dim()
    );
    println!(
        "    {} ghost install",
        style("$").dim()
    );

    Ok(())
}
