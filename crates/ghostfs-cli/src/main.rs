//! # ghost CLI
//!
//! The `ghost` command — GhostFS dependency virtualization engine.

mod commands;

use clap::{Parser, Subcommand};
use console::style;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "ghost",
    version,
    about = "👻 GhostFS — Dependency Virtualization Engine",
    long_about = "GhostFS eliminates node_modules by virtualizing dependencies.\n\
                  Packages are stored once globally and resolved at runtime.\n\
                  No install. No duplication. No waiting."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new GhostFS project (creates ghost.json)
    Init,

    /// Create a project from a template (react, next, vite, node)
    Create {
        /// Template name (react, next, vite, node)
        template: String,

        /// Project name / directory
        name: String,
    },

    /// Add a package to dependencies
    Add {
        /// Package name (e.g. "react" or "react@19.0.0")
        package: String,

        /// Add as devDependency
        #[arg(short = 'D', long)]
        dev: bool,
    },

    /// Remove a package from dependencies
    Remove {
        /// Package name to remove
        package: String,
    },

    /// Install all dependencies to the global store
    Install {
        /// Also create node_modules/ with symlinks (compatibility mode)
        #[arg(long)]
        link: bool,
    },

    /// Link packages from global store into node_modules/
    Link,

    /// Remove managed node_modules/
    Unlink,

    /// List packages in the global store
    List,

    /// Run a script defined in ghost.json
    Run {
        /// Script name to run
        script: String,

        /// Additional arguments passed to the script
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },

    /// Shortcut for `ghost run dev`
    Dev {
        /// Additional arguments passed to the dev script
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },

    /// Show store statistics
    Status,

    /// Clean the global package store
    Clean {
        /// Skip confirmation
        #[arg(long)]
        force: bool,
    },

    /// List available project templates
    Templates,

    /// Install Node.js runtime hooks to allow zero-node_modules resolution
    InstallHooks,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .without_time()
        .init();

    let cli = Cli::parse();
    let project_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    let result = match cli.command {
        Commands::Init => commands::init::run(&project_dir),
        Commands::Create { template, name } => {
            commands::create::run(&project_dir, &template, &name)
        }
        Commands::Add { package, dev } => commands::add::run(&project_dir, &package, dev).await,
        Commands::Remove { package } => commands::remove::run(&project_dir, &package),
        Commands::Install { link } => {
            let r = commands::install::run(&project_dir).await;
            if r.is_ok() && link {
                if let Err(e) = commands::link::run(&project_dir) {
                    eprintln!("{} Link failed: {:#}", style("warning:").yellow(), e);
                }
            }
            r
        }
        Commands::Link => commands::link::run(&project_dir),
        Commands::Unlink => commands::link::unlink(&project_dir),
        Commands::List => commands::list::run(),
        Commands::Run { script, args } => commands::run::run(&project_dir, &script, &args),
        Commands::Dev { args } => commands::run::run(&project_dir, "dev", &args),
        Commands::Status => print_status(),
        Commands::Clean { force } => commands::clean::run(force),
        Commands::Templates => commands::create::list_templates(),
        Commands::InstallHooks => commands::hooks::run(),
    };

    if let Err(e) = result {
        eprintln!("{} {:#}", style("error:").red().bold(), e);
        std::process::exit(1);
    }
}

fn print_status() -> anyhow::Result<()> {
    let store = ghostfs_store::ContentStore::new()?;
    let count = store.db().package_count()?;
    let total_size = store.db().total_size()?;

    println!("{}", style("👻 GhostFS Status").bold());
    println!();
    println!(
        "  {} {}",
        style("Store:").dim(),
        store.root().display()
    );
    println!(
        "  {} {}",
        style("Packages:").dim(),
        count
    );
    println!(
        "  {} {}",
        style("Total size:").dim(),
        format_size(total_size)
    );
    println!();
    println!(
        "  {} {}",
        style("Runtime hooks:").dim(),
        runtime_hooks_path()
    );
    Ok(())
}

fn runtime_hooks_path() -> String {
    if let Some(home) = dirs::home_dir() {
        let resolver = home.join(".ghostfs").join("runtime").join("resolver.js");
        if resolver.exists() {
            return resolver.to_string_lossy().to_string();
        }
    }
    "not installed (run ghost install-hooks)".to_string()
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
