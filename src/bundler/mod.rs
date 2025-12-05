//! Core bundler implementation
//!
//! Handles the module graph, dependency resolution, and bundle generation.

mod graph;
mod chunk;

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use anyhow::{Context, Result};
use parking_lot::RwLock;
use sha2::{Sha256, Digest};
use tracing::{debug, info};

use crate::cli::BuildOptions;
use crate::config::Config;
use crate::resolver::Resolver;
use crate::transform::Transformer;

pub use graph::{ModuleGraph, Module, ModuleId, ModuleType};
pub use chunk::{Chunk, ChunkType};

/// Result of a build operation
#[derive(Debug)]
pub struct BuildResult {
    /// Generated bundles
    pub bundles: Vec<BundleInfo>,
    
    /// Asset manifest
    pub manifest: HashMap<String, String>,
}

/// Information about a generated bundle
#[derive(Debug)]
pub struct BundleInfo {
    /// Output file path
    pub output_path: PathBuf,
    
    /// Bundle size in bytes
    pub size: usize,
    
    /// Source map path (if generated)
    pub sourcemap_path: Option<PathBuf>,
}

/// The main bundler
pub struct Bundler {
    /// Project configuration
    config: Arc<Config>,
    
    /// Build options
    options: BuildOptions,
    
    /// Module resolver
    resolver: Resolver,
    
    /// Code transformer
    transformer: Transformer,
    
    /// Module graph
    graph: Arc<RwLock<ModuleGraph>>,
}

impl Bundler {
    /// Create a new bundler instance
    pub fn new(config: Config, options: BuildOptions) -> Result<Self> {
        let config = Arc::new(config);
        let resolver = Resolver::new(config.clone())?;
        let transformer = Transformer::new(config.clone())?;
        
        Ok(Self {
            config,
            options,
            resolver,
            transformer,
            graph: Arc::new(RwLock::new(ModuleGraph::new())),
        })
    }
    
    /// Build the project
    pub async fn build(&self) -> Result<BuildResult> {
        let start = Instant::now();
        
        // 1. Build the module graph from entrypoints
        info!("Building module graph...");
        self.build_module_graph().await?;
        
        // 2. Transform all modules
        info!("Transforming modules...");
        self.transform_modules().await?;
        
        // 3. Generate chunks
        info!("Generating chunks...");
        let chunks = self.generate_chunks()?;
        
        // 4. Write output bundles
        info!("Writing bundles...");
        let bundles = self.write_bundles(&chunks)?;
        
        // 5. Generate manifest
        let manifest = self.generate_manifest(&bundles)?;
        
        debug!("Build completed in {:?}", start.elapsed());
        
        Ok(BuildResult { bundles, manifest })
    }
    
    /// Build the module graph by traversing from entrypoints
    async fn build_module_graph(&self) -> Result<()> {
        let entrypoints = self.config.all_entrypoints();
        
        for (name, path) in entrypoints {
            debug!("Processing entrypoint: {} -> {}", name, path.display());
            self.process_module(&path, true).await?;
        }
        
        Ok(())
    }
    
    /// Process a single module and its dependencies
    /// 
    /// Uses Box::pin for async recursion to avoid infinite type size issues
    async fn process_module(&self, path: &PathBuf, is_entry: bool) -> Result<ModuleId> {
        let canonical_path = fs::canonicalize(path)
            .with_context(|| format!("Failed to resolve module path: {}", path.display()))?;
        
        // Check if already processed
        {
            let graph = self.graph.read();
            if let Some(id) = graph.get_module_id(&canonical_path) {
                return Ok(id);
            }
        }
        
        // Read module source
        let source = fs::read_to_string(&canonical_path)
            .with_context(|| format!("Failed to read module: {}", canonical_path.display()))?;
        
        // Determine module type from extension
        let module_type = Module::detect_type(&canonical_path);
        
        // Parse and extract dependencies
        let dependencies = self.resolver.extract_dependencies(&source, &canonical_path, &module_type)?;
        
        // Create module
        let module = Module {
            path: canonical_path.clone(),
            source,
            module_type,
            is_entry,
            dependencies: dependencies.clone(),
            transformed: None,
        };
        
        // Add to graph
        let module_id = {
            let mut graph = self.graph.write();
            graph.add_module(module)
        };
        
        // Process dependencies recursively (Box::pin needed for async recursion)
        for dep in dependencies {
            let resolved = self.resolver.resolve(&dep, &canonical_path)?;
            if let Some(resolved_path) = resolved {
                let dep_id = Box::pin(self.process_module(&resolved_path, false)).await?;
                
                let mut graph = self.graph.write();
                graph.add_dependency(module_id, dep_id);
            }
        }
        
        Ok(module_id)
    }
    
    /// Transform all modules in the graph
    async fn transform_modules(&self) -> Result<()> {
        let module_ids: Vec<ModuleId> = {
            let graph = self.graph.read();
            graph.all_module_ids()
        };
        
        for id in module_ids {
            let (source, path, module_type) = {
                let graph = self.graph.read();
                let module = graph.get_module(id).unwrap();
                (module.source.clone(), module.path.clone(), module.module_type.clone())
            };
            
            let transformed = self.transformer.transform(&source, &path, &module_type)?;
            
            {
                let mut graph = self.graph.write();
                if let Some(module) = graph.get_module_mut(id) {
                    module.transformed = Some(transformed);
                }
            }
        }
        
        Ok(())
    }
    
    /// Generate chunks from the module graph
    fn generate_chunks(&self) -> Result<Vec<Chunk>> {
        let graph = self.graph.read();
        
        // For Milestone 1: single chunk per entrypoint
        let mut chunks = Vec::new();
        
        for (name, path) in self.config.all_entrypoints() {
            let canonical_path = fs::canonicalize(&path)?;
            
            if let Some(entry_id) = graph.get_module_id(&canonical_path) {
                // Get all modules reachable from this entry
                let module_ids = graph.get_reachable_modules(entry_id);
                
                chunks.push(Chunk {
                    name,
                    chunk_type: ChunkType::Entry,
                    module_ids,
                });
            }
        }
        
        Ok(chunks)
    }
    
    /// Write bundles to disk
    fn write_bundles(&self, chunks: &[Chunk]) -> Result<Vec<BundleInfo>> {
        let output_dir = self.options.outdir.clone()
            .unwrap_or_else(|| self.config.output_dir());
        
        fs::create_dir_all(&output_dir)
            .context("Failed to create output directory")?;
        
        let graph = self.graph.read();
        let mut bundles = Vec::new();
        
        for chunk in chunks {
            // Concatenate all transformed module code
            let mut bundle_code = String::new();
            
            // Add runtime header
            bundle_code.push_str(&self.generate_runtime_header());
            
            for &module_id in &chunk.module_ids {
                if let Some(module) = graph.get_module(module_id) {
                    let code = module.transformed.as_ref()
                        .unwrap_or(&module.source);
                    
                    // Wrap module in a function
                    bundle_code.push_str(&format!(
                        "\n// Module: {}\n__component_modules__[\"{}\"] = function(module, exports, require) {{\n{}\n}};\n",
                        module.path.display(),
                        module.path.display(),
                        code
                    ));
                }
            }
            
            // Add entry point execution
            if let ChunkType::Entry = chunk.chunk_type {
                if let Some(&entry_id) = chunk.module_ids.first() {
                    if let Some(entry_module) = graph.get_module(entry_id) {
                        bundle_code.push_str(&format!(
                            "\n// Execute entry point\n__component_require__(\"{}\");\n",
                            entry_module.path.display()
                        ));
                    }
                }
            }
            
            // Minify if enabled
            let final_code = if self.options.minify {
                self.minify_code(&bundle_code)?
            } else {
                bundle_code
            };
            
            // Generate hash for filename
            let hash = if self.config.output.hash {
                let mut hasher = Sha256::new();
                hasher.update(final_code.as_bytes());
                let result = hasher.finalize();
                format!(".{}", &hex::encode(result)[..8])
            } else {
                String::new()
            };
            
            // Write bundle
            let filename = format!("{}{}.js", chunk.name, hash);
            let output_path = output_dir.join(&filename);
            
            fs::write(&output_path, &final_code)
                .with_context(|| format!("Failed to write bundle: {}", output_path.display()))?;
            
            bundles.push(BundleInfo {
                output_path,
                size: final_code.len(),
                sourcemap_path: None, // TODO: Generate sourcemaps
            });
        }
        
        Ok(bundles)
    }
    
    /// Generate the module runtime header
    fn generate_runtime_header(&self) -> String {
        r#"// Component Runtime
(function() {
  var __component_modules__ = {};
  var __component_cache__ = {};
  
  function __component_require__(moduleId) {
    if (__component_cache__[moduleId]) {
      return __component_cache__[moduleId].exports;
    }
    
    var module = { exports: {} };
    __component_cache__[moduleId] = module;
    
    var moduleFn = __component_modules__[moduleId];
    if (moduleFn) {
      moduleFn(module, module.exports, __component_require__);
    }
    
    return module.exports;
  }
  
  window.__component_modules__ = __component_modules__;
  window.__component_require__ = __component_require__;
})();
"#.to_string()
    }
    
    /// Minify JavaScript code (basic implementation)
    fn minify_code(&self, code: &str) -> Result<String> {
        // For now, just remove extra whitespace and comments
        // In a full implementation, we'd use swc minifier
        let mut result = String::with_capacity(code.len());
        let mut in_string = false;
        let mut string_char = ' ';
        let mut in_single_comment = false;
        let mut in_multi_comment = false;
        let mut prev_char = ' ';
        let mut chars = code.chars().peekable();
        
        while let Some(c) = chars.next() {
            if in_single_comment {
                if c == '\n' {
                    in_single_comment = false;
                    result.push('\n');
                }
                continue;
            }
            
            if in_multi_comment {
                if prev_char == '*' && c == '/' {
                    in_multi_comment = false;
                }
                prev_char = c;
                continue;
            }
            
            if in_string {
                result.push(c);
                if c == string_char && prev_char != '\\' {
                    in_string = false;
                }
                prev_char = c;
                continue;
            }
            
            if c == '"' || c == '\'' || c == '`' {
                in_string = true;
                string_char = c;
                result.push(c);
                prev_char = c;
                continue;
            }
            
            if c == '/' {
                if let Some(&next) = chars.peek() {
                    if next == '/' {
                        in_single_comment = true;
                        chars.next();
                        continue;
                    } else if next == '*' {
                        in_multi_comment = true;
                        chars.next();
                        continue;
                    }
                }
            }
            
            // Collapse whitespace
            if c.is_whitespace() {
                if !result.ends_with(' ') && !result.ends_with('\n') {
                    result.push(' ');
                }
            } else {
                result.push(c);
            }
            
            prev_char = c;
        }
        
        Ok(result)
    }
    
    /// Generate asset manifest
    fn generate_manifest(&self, bundles: &[BundleInfo]) -> Result<HashMap<String, String>> {
        let mut manifest = HashMap::new();
        
        for bundle in bundles {
            if let Some(filename) = bundle.output_path.file_name() {
                let name = filename.to_string_lossy().to_string();
                manifest.insert(name.clone(), name);
            }
        }
        
        // Write manifest file if enabled
        if self.config.output.manifest {
            let output_dir = self.options.outdir.clone()
                .unwrap_or_else(|| self.config.output_dir());
            let manifest_path = output_dir.join("manifest.json");
            
            let manifest_json = serde_json::to_string_pretty(&manifest)?;
            fs::write(&manifest_path, manifest_json)
                .context("Failed to write manifest.json")?;
        }
        
        Ok(manifest)
    }
}
