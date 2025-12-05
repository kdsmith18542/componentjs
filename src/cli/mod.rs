//! Command-line interface for Component
//!
//! Provides the main CLI structure using clap with subcommands for:
//! - `build`: Production build
//! - `dev`: Development server with HMR
//! - `init`: Project scaffolding

mod build;
mod dev;
mod init;

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;

pub use build::{BuildCommand, BuildOptions};
pub use dev::{DevCommand, DevServerOptions};
pub use init::InitCommand;

/// Component Reborn - A modern, batteries-included frontend build tool
#[derive(Parser, Debug)]
#[command(name = "component")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Path to component.toml config file
    #[arg(short, long, global = true, default_value = "component.toml")]
    pub config: String,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Build the project for production
    Build(BuildCommand),

    /// Start development server with hot module replacement
    Dev(DevCommand),

    /// Initialize a new project
    Init(InitCommand),
}

impl Cli {
    /// Execute the CLI command
    pub async fn execute(&self) -> Result<()> {
        print_banner();

        match &self.command {
            Commands::Build(cmd) => cmd.execute(&self.config).await,
            Commands::Dev(cmd) => cmd.execute(&self.config).await,
            Commands::Init(cmd) => cmd.execute().await,
        }
    }
}

/// Print the Component banner
fn print_banner() {
    eprintln!(
        "\n{} {} {}\n",
        "⚡".cyan(),
        "Component".bold().cyan(),
        format!("v{}", env!("CARGO_PKG_VERSION")).dimmed()
    );
}
