use anyhow::{Context, Result};
use console::style;
use std::fs;

// Embed the JavaScript/ESM runtime hook files directly into the CLI binary
const RESOLVER_JS: &str = include_str!("../../../../runtime/resolver.js");
const LOADER_MJS: &str = include_str!("../../../../runtime/loader.mjs");

/// Installs the runtime hook files into ~/.ghostfs/runtime/
pub fn run() -> Result<()> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    let runtime_dir = home.join(".ghostfs").join("runtime");

    fs::create_dir_all(&runtime_dir)
        .with_context(|| format!("Failed to create runtime directory: {}", runtime_dir.display()))?;

    let resolver_path = runtime_dir.join("resolver.js");
    let loader_path = runtime_dir.join("loader.mjs");

    fs::write(&resolver_path, RESOLVER_JS)
        .with_context(|| format!("Failed to write resolver hook to {}", resolver_path.display()))?;

    fs::write(&loader_path, LOADER_MJS)
        .with_context(|| format!("Failed to write loader hook to {}", loader_path.display()))?;

    println!(
        "{} Runtime hooks installed successfully!",
        style("✓").green().bold()
    );
    println!("  {} {}", style("Resolver (CJS):").dim(), resolver_path.display());
    println!("  {} {}", style("Loader (ESM):").dim(), loader_path.display());
    println!();
    println!("  To use them manually with Node.js, run:");
    if cfg!(target_os = "windows") {
        println!("    $env:NODE_OPTIONS=\"--require `\"{}\"\"\"", resolver_path.display());
    } else {
        println!("    export NODE_OPTIONS=\"--require {}\"", resolver_path.display());
    }
    println!();

    Ok(())
}
