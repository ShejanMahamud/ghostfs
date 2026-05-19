use anyhow::{Context, Result};
use console::style;
use ghostfs_core::{Lockfile, Manifest};
use ghostfs_store::ContentStore;
use std::path::Path;
use std::process::Command;

/// Run a script defined in ghost.json with GhostFS resolution.
pub fn run(project_dir: &Path, script: &str, args: &[String]) -> Result<()> {
    let manifest = Manifest::load_from_dir(project_dir)?;

    // Look up the script
    let cmd = manifest
        .scripts
        .get(script)
        .with_context(|| format!("Script '{}' not found in ghost.json", script))?;

    println!(
        "{} Running script '{}': {}",
        style("▶").cyan().bold(),
        style(script).green(),
        style(cmd).dim()
    );

    // Build NODE_PATH from the global store
    let node_path = build_node_path(project_dir)?;

    // Check for resolver hook
    let resolver_path = find_resolver_hook();

    // Build NODE_OPTIONS with resolver hook if available
    let mut node_options = std::env::var("NODE_OPTIONS").unwrap_or_default();
    if let Some(ref hook) = resolver_path {
        if !node_options.contains("ghostfs") {
            let hook_str = hook.to_string_lossy().replace('\\', "/");
            let require_flag = format!("--require \"{}\"", hook_str);
            if node_options.is_empty() {
                node_options = require_flag;
            } else {
                node_options = format!("{} {}", node_options, require_flag);
            }
        }
    }

    // Execute the script
    let mut child = Command::new(shell_program());
    child
        .args(shell_args(cmd))
        .args(args)
        .current_dir(project_dir)
        .env("NODE_PATH", &node_path)
        .env("GHOSTFS", "1");

    if !node_options.is_empty() {
        child.env("NODE_OPTIONS", &node_options);
    }

    let status = child
        .status()
        .with_context(|| format!("Failed to execute script '{}'", script))?;

    if !status.success() {
        let code = status.code().unwrap_or(1);
        anyhow::bail!("Script '{}' exited with code {}", script, code);
    }

    Ok(())
}

/// Build a NODE_PATH string pointing to every package in the global store.
fn build_node_path(project_dir: &Path) -> Result<String> {
    let lockfile_path = Lockfile::lockfile_path(project_dir);
    let store = ContentStore::new()?;
    let mut paths = Vec::new();

    if lockfile_path.exists() {
        let lockfile = Lockfile::load(&lockfile_path)?;
        for locked_pkg in lockfile.packages.values() {
            let pkg_path = store.package_path(&locked_pkg.hash);
            if pkg_path.exists() {
                paths.push(pkg_path.to_string_lossy().to_string());
            }
        }
    }

    let separator = if cfg!(target_os = "windows") { ";" } else { ":" };
    Ok(paths.join(separator))
}

/// Find the resolver hook in the standard location.
fn find_resolver_hook() -> Option<std::path::PathBuf> {
    // Check bundled runtime/ directory first
    let exe_dir = std::env::current_exe().ok()?.parent()?.to_path_buf();
    let bundled = exe_dir.join("runtime").join("resolver.js");
    if bundled.exists() {
        return Some(bundled);
    }

    // Check ~/.ghostfs/runtime/
    let home = dirs::home_dir()?;
    let global = home.join(".ghostfs").join("runtime").join("resolver.js");
    if global.exists() {
        return Some(global);
    }

    None
}

fn shell_program() -> &'static str {
    if cfg!(target_os = "windows") {
        "cmd"
    } else {
        "sh"
    }
}

fn shell_args(cmd: &str) -> Vec<&str> {
    if cfg!(target_os = "windows") {
        vec!["/C", cmd]
    } else {
        vec!["-c", cmd]
    }
}
