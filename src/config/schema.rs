//! Configuration schema definitions

use serde::{Deserialize, Serialize};

/// Project metadata configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// Project name
    pub name: String,
    
    /// Project version
    #[serde(default = "default_version")]
    pub version: String,
}

fn default_version() -> String {
    "0.1.0".to_string()
}

/// Output configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Output directory
    #[serde(default = "default_output_dir")]
    pub dir: String,
    
    /// Public URL prefix for assets
    #[serde(default = "default_public_url")]
    pub public_url: String,
    
    /// Hash assets for cache busting
    #[serde(default = "default_true")]
    pub hash: bool,
    
    /// Generate asset manifest
    #[serde(default = "default_true")]
    pub manifest: bool,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            dir: default_output_dir(),
            public_url: default_public_url(),
            hash: true,
            manifest: true,
        }
    }
}

fn default_output_dir() -> String {
    "dist".to_string()
}

fn default_public_url() -> String {
    "/".to_string()
}

fn default_true() -> bool {
    true
}

/// Feature flags configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeaturesConfig {
    /// Enable JSX transformation
    #[serde(default)]
    pub jsx: bool,
    
    /// JSX runtime (classic or automatic)
    #[serde(default = "default_jsx_runtime")]
    pub jsx_runtime: String,
    
    /// JSX import source for automatic runtime
    #[serde(default = "default_jsx_import_source")]
    pub jsx_import_source: String,
    
    /// Enable TypeScript
    #[serde(default)]
    pub typescript: bool,
    
    /// Enable CSS modules
    #[serde(default)]
    pub css_modules: bool,
    
    /// CSS modules pattern for class names
    #[serde(default = "default_css_modules_pattern")]
    pub css_modules_pattern: String,
    
    /// Enable Tailwind CSS processing
    #[serde(default)]
    pub tailwind: bool,
    
    /// Enable tree shaking
    #[serde(default = "default_true")]
    pub tree_shaking: bool,
    
    /// Enable code splitting
    #[serde(default = "default_true")]
    pub code_splitting: bool,
}

impl Default for FeaturesConfig {
    fn default() -> Self {
        Self {
            jsx: false,
            jsx_runtime: default_jsx_runtime(),
            jsx_import_source: default_jsx_import_source(),
            typescript: false,
            css_modules: false,
            css_modules_pattern: default_css_modules_pattern(),
            tailwind: false,
            tree_shaking: true,
            code_splitting: true,
        }
    }
}

fn default_jsx_runtime() -> String {
    "automatic".to_string()
}

fn default_jsx_import_source() -> String {
    "react".to_string()
}

fn default_css_modules_pattern() -> String {
    "[name]__[local]__[hash:8]".to_string()
}

/// Development server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevConfig {
    /// Port to run dev server on
    #[serde(default = "default_port")]
    pub port: u16,
    
    /// Host to bind to
    #[serde(default = "default_host")]
    pub host: String,
    
    /// Open browser automatically
    #[serde(default)]
    pub open: bool,
    
    /// Enable hot module replacement
    #[serde(default = "default_true")]
    pub hmr: bool,
    
    /// Proxy configuration for API requests
    #[serde(default)]
    pub proxy: Vec<ProxyConfig>,
}

impl Default for DevConfig {
    fn default() -> Self {
        Self {
            port: default_port(),
            host: default_host(),
            open: false,
            hmr: true,
            proxy: Vec::new(),
        }
    }
}

fn default_port() -> u16 {
    3000
}

fn default_host() -> String {
    "localhost".to_string()
}

/// Proxy configuration for dev server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    /// Path prefix to proxy
    pub path: String,
    
    /// Target URL
    pub target: String,
    
    /// Rewrite path
    #[serde(default)]
    pub rewrite: Option<String>,
}

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// Plugin name/identifier
    pub name: String,
    
    /// Plugin-specific options
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub options: Option<toml::Table>,
}
