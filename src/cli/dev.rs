//! Development server command implementation

use std::sync::Arc;

use anyhow::Result;
use clap::Args;
use colored::Colorize;
use tracing::info;

use crate::config::Config;
use crate::server::DevServer;

/// Start development server with hot module replacement
#[derive(Args, Debug)]
pub struct DevCommand {
    /// Port to run the dev server on
    #[arg(short, long, default_value = "3000")]
    pub port: u16,

    /// Host to bind to
    #[arg(long, default_value = "localhost")]
    pub host: String,

    /// Open browser automatically
    #[arg(long)]
    pub open: bool,

    /// Disable hot module replacement
    #[arg(long)]
    pub no_hmr: bool,
}

impl DevCommand {
    pub async fn execute(&self, config_path: &str) -> Result<()> {
        info!("Loading configuration from {}", config_path);
        let config = Config::load(config_path)?;
        
        let addr = format!("{}:{}", self.host, self.port);
        
        eprintln!(
            "{} Starting dev server at {}\n",
            "→".blue(),
            format!("http://{}", addr).cyan().underline()
        );
        
        if !self.no_hmr {
            eprintln!(
                "  {} Hot Module Replacement {}",
                "•".dimmed(),
                "enabled".green()
            );
        }
        
        eprintln!(
            "  {} Press {} to stop\n",
            "•".dimmed(),
            "Ctrl+C".yellow()
        );
        
        let server = DevServer::new(Arc::new(config), DevServerOptions {
            host: self.host.clone(),
            port: self.port,
            hmr: !self.no_hmr,
            open: self.open,
        })?;
        
        server.start().await
    }
}

/// Development server options
#[derive(Debug, Clone)]
pub struct DevServerOptions {
    pub host: String,
    pub port: u16,
    pub hmr: bool,
    pub open: bool,
}
