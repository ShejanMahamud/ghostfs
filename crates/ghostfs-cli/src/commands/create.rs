use anyhow::Result;
use console::style;
use ghostfs_core::Scaffolder;
use std::path::Path;

/// Scaffold a new project from a template.
pub async fn run(project_dir: &Path, template: &str, name: &str) -> Result<()> {
    let is_builtin = matches!(template, "react" | "next" | "vite" | "node" | "basic");

    if is_builtin {
        let target = project_dir.join(name);

        if target.exists() {
            anyhow::bail!("Directory '{}' already exists", target.display());
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
        println!("    {} cd {}", style("$").dim(), name);
        println!("    {} ghost install", style("$").dim());
        println!("    {} ghost dev", style("$").dim());
        println!();
    } else {
        // Fallback: run create-<template> dynamically from the registry via dlx
        let package_name = if template.starts_with("create-") {
            template.to_string()
        } else {
            format!("create-{}", template)
        };

        println!(
            "{} Delegating to registry template initializer '{}'...",
            style("◌").blue(),
            style(&package_name).cyan()
        );

        super::dlx::run(&package_name, &[name.to_string()]).await?;
    }

    Ok(())
}

/// List available templates.
pub fn list_templates() -> Result<()> {
    println!("{} Available built-in templates:\n", style("📦").bold());

    for (name, desc) in Scaffolder::templates() {
        println!("  {:<10} {}", style(name).cyan().bold(), style(desc).dim());
    }

    println!();
    println!(
        "  Usage: {} ghost create <template> <name>",
        style("$").dim()
    );
    println!("  Note: You can also specify any external npm initializer template");
    println!("        (e.g., 'ghost create next-app my-app' runs 'create-next-app').");
    println!();

    Ok(())
}
