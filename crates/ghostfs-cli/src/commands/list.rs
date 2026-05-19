use anyhow::Result;
use console::style;
use ghostfs_store::ContentStore;

/// List all packages in the global GhostFS store.
pub fn run() -> Result<()> {
    let store = ContentStore::new()?;
    let packages = store.list_packages()?;

    if packages.is_empty() {
        println!("{} No packages in the global store yet.", style("ℹ").blue());
        println!("  Run {} to get started.", style("ghost add <pkg>").cyan());
        return Ok(());
    }

    println!(
        "{} Global Store — {} packages\n",
        style("👻").bold(),
        packages.len()
    );

    println!(
        "  {:<30} {:<15} {:<12} {}",
        style("Package").bold().underlined(),
        style("Version").bold().underlined(),
        style("Size").bold().underlined(),
        style("Hash").bold().underlined(),
    );

    for pkg in &packages {
        let size_str = format_size(pkg.size);
        let short_hash = &pkg.hash[..12.min(pkg.hash.len())];
        println!(
            "  {:<30} {:<15} {:<12} {}",
            style(&pkg.name).cyan(),
            style(&pkg.version).green(),
            style(&size_str).dim(),
            style(short_hash).dim(),
        );
    }

    let total_size: u64 = packages.iter().map(|p| p.size).sum();
    println!();
    println!(
        "  Total: {} in {} packages",
        style(format_size(total_size)).bold(),
        packages.len()
    );

    Ok(())
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}
