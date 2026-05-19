use anyhow::Result;
use console::style;
use ghostfs_core::Scaffolder;
use std::path::Path;

/// Scaffold a new project from a template.
pub fn run(project_dir: &Path, template: &str, name: &str) -> Result<()> {
    let target = project_dir.join(name);

    if target.exists() {
        anyhow::bail!(
            "Directory '{}' already exists",
            target.display()
        );
    }

    println!(
        "{} Creating {} project '{}'...",
        style("◌").blue(),
        style(template).cyan(),
        style(name).green()
    );

    Scaffolder::create(&target, template, name)?;

    println!();
    println!(
        "{} Project '{}' created!",
        style("✓").green().bold(),
        style(name).cyan()
    );
    println!();
    println!("  Get started:");
    println!(
        "    {} cd {}",
        style("$").dim(),
        name
    );
    println!(
        "    {} ghost install",
        style("$").dim()
    );
    println!(
        "    {} ghost dev",
        style("$").dim()
    );
    println!();

    Ok(())
}

/// List available templates.
pub fn list_templates() -> Result<()> {
    println!(
        "{} Available templates:\n",
        style("📦").bold()
    );

    for (name, desc) in Scaffolder::templates() {
        println!(
            "  {:<10} {}",
            style(name).cyan().bold(),
            style(desc).dim()
        );
    }

    println!();
    println!(
        "  Usage: {} ghost create <template> <name>",
        style("$").dim()
    );
    println!();

    Ok(())
}
