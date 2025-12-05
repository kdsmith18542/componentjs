//! Module resolution
//!
//! Handles resolving import specifiers to actual file paths.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use once_cell::sync::Lazy;
use regex::Regex;
use tracing::debug;

use crate::bundler::ModuleType;
use crate::config::Config;

/// Regex patterns for extracting imports
static IMPORT_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?:import|export)\s+(?:(?:\{[^}]*\}|\*\s+as\s+\w+|\w+)\s+from\s+)?["']([^"']+)["']|require\s*\(\s*["']([^"']+)["']\s*\)"#).unwrap()
});

static DYNAMIC_IMPORT_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"import\s*\(\s*["']([^"']+)["']\s*\)"#).unwrap()
});

/// Module resolver
pub struct Resolver {
    /// Project configuration
    #[allow(dead_code)]
    config: Arc<Config>,
}

impl Resolver {
    /// Create a new resolver
    pub fn new(config: Arc<Config>) -> Result<Self> {
        Ok(Self {
            config,
        })
    }
    
    /// Extract import/require dependencies from source code
    pub fn extract_dependencies(
        &self,
        source: &str,
        _file_path: &Path,
        module_type: &ModuleType,
    ) -> Result<Vec<String>> {
        // Skip non-JS modules for now
        if !module_type.is_js_like() {
            return Ok(Vec::new());
        }
        
        let mut dependencies = Vec::new();
        
        // Find static imports/exports
        for cap in IMPORT_REGEX.captures_iter(source) {
            if let Some(specifier) = cap.get(1).or_else(|| cap.get(2)) {
                let spec = specifier.as_str().to_string();
                if !dependencies.contains(&spec) {
                    dependencies.push(spec);
                }
            }
        }
        
        // Find dynamic imports
        for cap in DYNAMIC_IMPORT_REGEX.captures_iter(source) {
            if let Some(specifier) = cap.get(1) {
                let spec = specifier.as_str().to_string();
                if !dependencies.contains(&spec) {
                    dependencies.push(spec);
                }
            }
        }
        
        debug!("Found {} dependencies", dependencies.len());
        
        Ok(dependencies)
    }
    
    /// Resolve an import specifier to an absolute file path
    pub fn resolve(&self, specifier: &str, from: &Path) -> Result<Option<PathBuf>> {
        debug!("Resolving '{}' from '{}'", specifier, from.display());
        
        // Skip external packages for now (bare specifiers)
        if !specifier.starts_with('.') && !specifier.starts_with('/') {
            debug!("Skipping bare specifier: {}", specifier);
            return Ok(None);
        }
        
        let base_dir = from.parent().unwrap_or(Path::new("."));
        
        // Try to resolve the path
        let resolved = self.resolve_relative(specifier, base_dir)?;
        
        debug!("Resolved to: {:?}", resolved);
        
        Ok(resolved)
    }
    
    /// Resolve a relative import
    fn resolve_relative(&self, specifier: &str, base_dir: &Path) -> Result<Option<PathBuf>> {
        let target = base_dir.join(specifier);
        
        // Try exact path first
        if target.is_file() {
            return Ok(Some(target));
        }
        
        // Try adding extensions
        let extensions = ["js", "ts", "jsx", "tsx", "mjs", "cjs", "json"];
        for ext in &extensions {
            let with_ext = target.with_extension(ext);
            if with_ext.is_file() {
                return Ok(Some(with_ext));
            }
        }
        
        // Try as directory with index file
        if target.is_dir() {
            for ext in &extensions {
                let index = target.join(format!("index.{}", ext));
                if index.is_file() {
                    return Ok(Some(index));
                }
            }
        }
        
        // Not found
        Ok(None)
    }
    
    /// Resolve a bare import (from node_modules)
    #[allow(dead_code)]
    fn resolve_bare(&self, specifier: &str, from: &Path) -> Result<Option<PathBuf>> {
        let mut current = from.to_path_buf();
        
        // Walk up directory tree looking for node_modules
        loop {
            let node_modules = current.join("node_modules");
            
            if node_modules.is_dir() {
                // Try to resolve in this node_modules
                if let Some(resolved) = self.resolve_in_node_modules(&node_modules, specifier)? {
                    return Ok(Some(resolved));
                }
            }
            
            // Move to parent directory
            if !current.pop() {
                break;
            }
        }
        
        Ok(None)
    }
    
    /// Resolve a module within a node_modules directory
    fn resolve_in_node_modules(&self, node_modules: &Path, specifier: &str) -> Result<Option<PathBuf>> {
        // Split specifier into package name and subpath
        let (package_name, subpath) = if specifier.starts_with('@') {
            // Scoped package: @scope/name or @scope/name/subpath
            let parts: Vec<&str> = specifier.splitn(3, '/').collect();
            if parts.len() < 2 {
                return Ok(None);
            }
            let name = format!("{}/{}", parts[0], parts[1]);
            let sub = if parts.len() > 2 {
                Some(parts[2..].join("/"))
            } else {
                None
            };
            (name, sub)
        } else {
            // Regular package: name or name/subpath
            let parts: Vec<&str> = specifier.splitn(2, '/').collect();
            let name = parts[0].to_string();
            let sub = parts.get(1).map(|s| s.to_string());
            (name, sub)
        };
        
        let package_dir = node_modules.join(&package_name);
        
        if !package_dir.is_dir() {
            return Ok(None);
        }
        
        // If there's a subpath, resolve it directly
        if let Some(sub) = subpath {
            return self.resolve_relative(&sub, &package_dir);
        }
        
        // Otherwise, look at package.json for main/module entry
        let package_json = package_dir.join("package.json");
        
        if package_json.is_file() {
            let content = fs::read_to_string(&package_json)
                .context("Failed to read package.json")?;
            let pkg: serde_json::Value = serde_json::from_str(&content)
                .context("Failed to parse package.json")?;
            
            // Try module field first (ESM)
            if let Some(module) = pkg.get("module").and_then(|v| v.as_str()) {
                let module_path = package_dir.join(module);
                if module_path.is_file() {
                    return Ok(Some(module_path));
                }
            }
            
            // Then try main field
            if let Some(main) = pkg.get("main").and_then(|v| v.as_str()) {
                return self.resolve_relative(main, &package_dir);
            }
        }
        
        // Default to index.js
        self.resolve_relative("index.js", &package_dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_imports() {
        let source = r#"
            import foo from './foo';
            import { bar } from './bar.js';
            import * as baz from '../baz';
            export { qux } from './qux';
            const x = require('./x');
        "#;
        
        let config = Config::default_config();
        let resolver = Resolver::new(Arc::new(config)).unwrap();
        let deps = resolver.extract_dependencies(source, Path::new("/test.js"), &ModuleType::JavaScript).unwrap();
        
        assert!(deps.contains(&"./foo".to_string()));
        assert!(deps.contains(&"./bar.js".to_string()));
        assert!(deps.contains(&"../baz".to_string()));
        assert!(deps.contains(&"./qux".to_string()));
        assert!(deps.contains(&"./x".to_string()));
    }
    
    #[test]
    fn test_extract_dynamic_imports() {
        let source = r#"
            const module = import('./dynamic');
            const other = import("./other");
        "#;
        
        let config = Config::default_config();
        let resolver = Resolver::new(Arc::new(config)).unwrap();
        let deps = resolver.extract_dependencies(source, Path::new("/test.js"), &ModuleType::JavaScript).unwrap();
        
        assert!(deps.contains(&"./dynamic".to_string()));
        assert!(deps.contains(&"./other".to_string()));
    }
}
