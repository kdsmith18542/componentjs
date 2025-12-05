//! Configuration handling for Component
//!
//! Parses and manages component.toml configuration files.

mod schema;

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

pub use schema::*;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Project metadata
    pub project: ProjectConfig,
    
    /// Entry points for bundling
    #[serde(default)]
    pub entrypoints: HashMap<String, String>,
    
    /// Output configuration
    #[serde(default)]
    pub output: OutputConfig,
    
    /// Feature flags
    #[serde(default)]
    pub features: FeaturesConfig,
    
    /// Development server settings
    #[serde(default)]
    pub dev: DevConfig,
    
    /// Plugin configuration
    #[serde(default)]
    pub plugins: Vec<PluginConfig>,
    
    /// Root directory (computed from config file location)
    #[serde(skip)]
    pub root: PathBuf,
}

impl Config {
    /// Load configuration from a file path
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let canonical_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()?.join(path)
        };
        
        let content = fs::read_to_string(&canonical_path)
            .with_context(|| format!("Failed to read config file: {}", canonical_path.display()))?;
        
        let mut config: Config = toml::from_str(&content)
            .with_context(|| "Failed to parse component.toml")?;
        
        // Set root directory to the directory containing the config file
        config.root = canonical_path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));
        
        // Validate configuration
        config.validate()?;
        
        Ok(config)
    }
    
    /// Create a default configuration
    pub fn default_config() -> Self {
        Self {
            project: ProjectConfig {
                name: "my-app".to_string(),
                version: "0.1.0".to_string(),
            },
            entrypoints: {
                let mut map = HashMap::new();
                map.insert("main".to_string(), "src/main.js".to_string());
                map
            },
            output: OutputConfig::default(),
            features: FeaturesConfig::default(),
            dev: DevConfig::default(),
            plugins: Vec::new(),
            root: PathBuf::from("."),
        }
    }
    
    /// Validate the configuration
    fn validate(&self) -> Result<()> {
        // Ensure at least one entrypoint exists
        if self.entrypoints.is_empty() {
            anyhow::bail!("At least one entrypoint must be specified in component.toml");
        }
        
        // Validate entrypoint paths exist
        for (name, path) in &self.entrypoints {
            let full_path = self.root.join(path);
            if !full_path.exists() {
                anyhow::bail!(
                    "Entrypoint '{}' points to non-existent file: {}",
                    name,
                    full_path.display()
                );
            }
        }
        
        Ok(())
    }
    
    /// Get the absolute output directory path
    pub fn output_dir(&self) -> PathBuf {
        self.root.join(&self.output.dir)
    }
    
    /// Get absolute path for an entrypoint
    pub fn entrypoint_path(&self, name: &str) -> Option<PathBuf> {
        self.entrypoints.get(name).map(|p| self.root.join(p))
    }
    
    /// Get all entrypoint paths
    pub fn all_entrypoints(&self) -> Vec<(String, PathBuf)> {
        self.entrypoints
            .iter()
            .map(|(name, path)| (name.clone(), self.root.join(path)))
            .collect()
    }
}
