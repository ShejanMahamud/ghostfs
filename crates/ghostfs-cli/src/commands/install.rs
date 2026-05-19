use anyhow::Result;
use console::style;
use ghostfs_core::Installer;
use ghostfs_registry::NpmRegistryClient;
use ghostfs_store::ContentStore;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;
use std::time::Instant;

/// Install all dependencies from ghost.json into the global store.
pub async fn run(project_dir: &Path) -> Result<()> {
    let start = Instant::now();

    println!("{} Resolving dependency tree...", style("◌").blue());

    let store = ContentStore::new()?;
    let registry = NpmRegistryClient::new();
    let installer = Installer::new(store, registry);

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{spinner:.cyan} {msg}")
            .unwrap()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
    );
    pb.set_message("Installing packages...");
    pb.enable_steady_tick(std::time::Duration::from_millis(80));

    let result = installer.install(project_dir).await?;

    pb.finish_and_clear();

    let elapsed = start.elapsed();
    println!();
    println!(
        "{} Done in {:.1}s",
        style("✓").green().bold(),
        elapsed.as_secs_f64()
    );
    println!();
    println!(
        "  {} {} total packages",
        style("packages").dim(),
        result.total
    );
    println!(
        "  {} {} downloaded, {} from cache",
        style("network").dim(),
        result.installed,
        result.cached
    );
    println!();
    println!(
        "  {} No node_modules created — dependencies live in the global store",
        style("👻").bold()
    );

    Ok(())
}
