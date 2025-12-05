//! Component Reborn - A modern, batteries-included frontend build tool
//!
//! This is a complete rewrite of the original componentjs package manager,
//! reimagined as a modern frontend build tool written in Rust.
//!
//! # Features
//! - ES modules, TypeScript, JSX/TSX support
//! - CSS modules, PostCSS integration
//! - Fast incremental compilation & HMR
//! - Dev server with WebSocket-based hot module replacement
//! - Plugin system for extensibility

use anyhow::Result;
use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod cli;
mod config;
mod bundler;
mod resolver;
mod transform;
mod server;
mod plugins;
mod utils;

pub use cli::Cli;
pub use config::Config;
pub use bundler::Bundler;

/// Initialize the logging/tracing system
fn init_tracing(verbose: bool) {
    let filter = if verbose {
        EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("component=debug,tower_http=debug"))
    } else {
        EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("component=info"))
    };

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    init_tracing(cli.verbose);
    
    cli.execute().await
}
