use anyhow::Result;
use console::style;
use ghostfs_core::Linker;
use ghostfs_store::ContentStore;
use std::path::Path;

/// Link packages from global store into node_modules/.
pub fn run(project_dir: &Path) -> Result<()> {
    println!("{} Linking packages to node_modules/...", style("◌").blue());

    let store = ContentStore::new()?;
    let linker = Linker::new(store);
    let result = linker.link(project_dir)?;

    if !result.errors.is_empty() {
        for err in &result.errors {
            eprintln!("  {} {}", style("⚠").yellow(), err);
        }
    }

    println!(
        "{} Linked {} packages into node_modules/",
        style("✓").green().bold(),
        result.linked
    );
    println!(
        "  {} node_modules is managed by GhostFS (symlinks to global store)",
        style("👻").bold()
    );

    Ok(())
}

/// Unlink — remove the managed node_modules.
pub fn unlink(project_dir: &Path) -> Result<()> {
    let store = ContentStore::new()?;
    let linker = Linker::new(store);
    linker.unlink(project_dir)?;

    println!(
        "{} Removed managed node_modules/",
        style("✓").green().bold()
    );

    Ok(())
}
