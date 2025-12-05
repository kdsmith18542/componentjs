//! Build command implementation

use std::path::PathBuf;
use std::time::Instant;

use anyhow::Result;
use clap::Args;
use colored::Colorize;
use tracing::info;

use crate::config::Config;
use crate::bundler::Bundler;

/// Build the project for production
#[derive(Args, Debug)]
pub struct BuildCommand {
    /// Output directory
    #[arg(short, long)]
    pub outdir: Option<PathBuf>,

    /// Enable minification
    #[arg(short, long, default_value = "true")]
    pub minify: bool,

    /// Enable source maps
    #[arg(long, default_value = "true")]
    pub sourcemap: bool,

    /// Target environment (es2020, es2021, es2022, esnext)
    #[arg(long, default_value = "es2020")]
    pub target: String,
}

impl BuildCommand {
    pub async fn execute(&self, config_path: &str) -> Result<()> {
        let start = Instant::now();
        
        info!("Loading configuration from {}", config_path);
        let config = Config::load(config_path)?;
        
        eprintln!("{} Building project...", "→".blue());
        
        let bundler = Bundler::new(config, self.into())?;
        let result = bundler.build().await?;
        
        let duration = start.elapsed();
        
        eprintln!(
            "\n{} Built {} bundle(s) in {:.2}s\n",
            "✓".green().bold(),
            result.bundles.len(),
            duration.as_secs_f64()
        );
        
        // Print bundle summary
        for bundle in &result.bundles {
            let size_kb = bundle.size as f64 / 1024.0;
            let size_str = if size_kb > 1024.0 {
                format!("{:.2} MB", size_kb / 1024.0)
            } else {
                format!("{:.2} KB", size_kb)
            };
            
            eprintln!(
                "  {} {} {}",
                "•".dimmed(),
                bundle.output_path.display().to_string().cyan(),
                size_str.dimmed()
            );
        }
        
        eprintln!();
        
        Ok(())
    }
}

/// Build options derived from command arguments
#[derive(Debug, Clone)]
pub struct BuildOptions {
    pub outdir: Option<PathBuf>,
    pub minify: bool,
    pub sourcemap: bool,
    pub target: String,
}

impl From<&BuildCommand> for BuildOptions {
    fn from(cmd: &BuildCommand) -> Self {
        Self {
            outdir: cmd.outdir.clone(),
            minify: cmd.minify,
            sourcemap: cmd.sourcemap,
            target: cmd.target.clone(),
        }
    }
}
