//! Plugin system for Component
//!
//! Provides a Vite/Rollup-style plugin API for extending the bundler.

use std::path::Path;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;

/// Plugin hook context
pub struct PluginContext {
    /// Project root directory
    pub root: std::path::PathBuf,
}

/// Result of a resolve hook
pub enum ResolveResult {
    /// Continue to next plugin
    Skip,
    /// Resolved path
    Resolved(String),
    /// Mark as external (don't bundle)
    External,
}

/// Result of a load hook
pub enum LoadResult {
    /// Continue to next plugin
    Skip,
    /// Loaded content
    Loaded {
        content: String,
        /// Optional loader type (js, css, json, etc.)
        loader: Option<String>,
    },
}

/// Result of a transform hook
pub enum TransformResult {
    /// Continue to next plugin (no transformation)
    Skip,
    /// Transformed code
    Transformed {
        code: String,
        /// Optional source map
        map: Option<String>,
    },
}

/// Plugin trait - implement this to create a Component plugin
#[async_trait]
pub trait Plugin: Send + Sync {
    /// Plugin name for logging and debugging
    fn name(&self) -> &str;
    
    /// Called when the build starts
    async fn build_start(&self, _ctx: &PluginContext) -> Result<()> {
        Ok(())
    }
    
    /// Called when the build ends
    async fn build_end(&self, _ctx: &PluginContext) -> Result<()> {
        Ok(())
    }
    
    /// Resolve an import specifier to a path
    /// Return ResolveResult::Skip to let other plugins handle it
    async fn resolve_id(
        &self,
        _specifier: &str,
        _importer: Option<&Path>,
        _ctx: &PluginContext,
    ) -> Result<ResolveResult> {
        Ok(ResolveResult::Skip)
    }
    
    /// Load the content of a module
    /// Return LoadResult::Skip to let other plugins handle it
    async fn load(
        &self,
        _id: &str,
        _ctx: &PluginContext,
    ) -> Result<LoadResult> {
        Ok(LoadResult::Skip)
    }
    
    /// Transform the code of a module
    /// Return TransformResult::Skip to leave code unchanged
    async fn transform(
        &self,
        _code: &str,
        _id: &str,
        _ctx: &PluginContext,
    ) -> Result<TransformResult> {
        Ok(TransformResult::Skip)
    }
}

/// Plugin manager
pub struct PluginManager {
    plugins: Vec<Arc<dyn Plugin>>,
    context: PluginContext,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new(root: std::path::PathBuf) -> Self {
        Self {
            plugins: Vec::new(),
            context: PluginContext { root },
        }
    }
    
    /// Register a plugin
    pub fn register(&mut self, plugin: Arc<dyn Plugin>) {
        self.plugins.push(plugin);
    }
    
    /// Run build_start hooks
    pub async fn run_build_start(&self) -> Result<()> {
        for plugin in &self.plugins {
            plugin.build_start(&self.context).await?;
        }
        Ok(())
    }
    
    /// Run build_end hooks
    pub async fn run_build_end(&self) -> Result<()> {
        for plugin in &self.plugins {
            plugin.build_end(&self.context).await?;
        }
        Ok(())
    }
    
    /// Run resolve_id hooks
    pub async fn resolve_id(
        &self,
        specifier: &str,
        importer: Option<&Path>,
    ) -> Result<Option<String>> {
        for plugin in &self.plugins {
            match plugin.resolve_id(specifier, importer, &self.context).await? {
                ResolveResult::Skip => continue,
                ResolveResult::Resolved(path) => return Ok(Some(path)),
                ResolveResult::External => return Ok(None),
            }
        }
        Ok(None)
    }
    
    /// Run load hooks
    pub async fn load(&self, id: &str) -> Result<Option<(String, Option<String>)>> {
        for plugin in &self.plugins {
            match plugin.load(id, &self.context).await? {
                LoadResult::Skip => continue,
                LoadResult::Loaded { content, loader } => {
                    return Ok(Some((content, loader)));
                }
            }
        }
        Ok(None)
    }
    
    /// Run transform hooks
    pub async fn transform(&self, code: &str, id: &str) -> Result<(String, Option<String>)> {
        let mut current_code = code.to_string();
        let mut current_map = None;
        
        for plugin in &self.plugins {
            match plugin.transform(&current_code, id, &self.context).await? {
                TransformResult::Skip => continue,
                TransformResult::Transformed { code, map } => {
                    current_code = code;
                    if map.is_some() {
                        current_map = map;
                    }
                }
            }
        }
        
        Ok((current_code, current_map))
    }
}

// Example built-in plugins

/// JSON plugin - transforms JSON files to ES modules
pub struct JsonPlugin;

#[async_trait]
impl Plugin for JsonPlugin {
    fn name(&self) -> &str {
        "json"
    }
    
    async fn transform(
        &self,
        code: &str,
        id: &str,
        _ctx: &PluginContext,
    ) -> Result<TransformResult> {
        if !id.ends_with(".json") {
            return Ok(TransformResult::Skip);
        }
        
        // Validate JSON
        serde_json::from_str::<serde_json::Value>(code)?;
        
        Ok(TransformResult::Transformed {
            code: format!("export default {};", code),
            map: None,
        })
    }
}

/// Virtual module plugin - allows defining virtual modules
pub struct VirtualPlugin {
    modules: std::collections::HashMap<String, String>,
}

impl VirtualPlugin {
    pub fn new() -> Self {
        Self {
            modules: std::collections::HashMap::new(),
        }
    }
    
    pub fn add_module(&mut self, id: &str, content: &str) {
        self.modules.insert(id.to_string(), content.to_string());
    }
}

impl Default for VirtualPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for VirtualPlugin {
    fn name(&self) -> &str {
        "virtual"
    }
    
    async fn resolve_id(
        &self,
        specifier: &str,
        _importer: Option<&Path>,
        _ctx: &PluginContext,
    ) -> Result<ResolveResult> {
        if self.modules.contains_key(specifier) {
            Ok(ResolveResult::Resolved(format!("\0virtual:{}", specifier)))
        } else {
            Ok(ResolveResult::Skip)
        }
    }
    
    async fn load(
        &self,
        id: &str,
        _ctx: &PluginContext,
    ) -> Result<LoadResult> {
        if let Some(stripped) = id.strip_prefix("\0virtual:") {
            if let Some(content) = self.modules.get(stripped) {
                return Ok(LoadResult::Loaded {
                    content: content.clone(),
                    loader: Some("js".to_string()),
                });
            }
        }
        Ok(LoadResult::Skip)
    }
}
