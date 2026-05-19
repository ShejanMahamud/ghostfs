use anyhow::Result;
use console::style;
use ghostfs_store::ContentStore;

/// Clean the global store — remove all cached packages.
pub fn run(force: bool) -> Result<()> {
    let store = ContentStore::new()?;
    let packages = store.list_packages()?;

    if packages.is_empty() {
        println!(
            "{} Store is already empty.",
            style("ℹ").blue()
        );
        return Ok(());
    }

    let total_size: u64 = packages.iter().map(|p| p.size).sum();

    if !force {
        println!(
            "{} This will remove {} packages ({}) from the global store.",
            style("⚠").yellow().bold(),
            packages.len(),
            format_size(total_size)
        );
        println!(
            "  Run with {} to confirm.",
            style("--force").cyan()
        );
        return Ok(());
    }

    println!(
        "{} Cleaning global store...",
        style("◌").blue()
    );

    // Remove shard directories in store root (e.g. "00" through "ff")
    let store_root = ContentStore::default_store_path()?;
    if store_root.exists() {
        for entry in std::fs::read_dir(&store_root)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    // Shards are 2 characters long (hexadecimal)
                    if name.len() == 2 {
                        if let Err(e) = std::fs::remove_dir_all(&path) {
                            eprintln!(
                                "  {} Failed to remove shard directory {}: {}",
                                style("⚠").yellow(),
                                name,
                                e
                            );
                        }
                    }
                }
            }
        }
    }

    // Clear packages metadata from database
    store.db().clear_packages()?;

    println!(
        "{} Removed {} packages (freed {})",
        style("✓").green().bold(),
        packages.len(),
        format_size(total_size)
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
