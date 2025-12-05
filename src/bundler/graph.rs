//! Module graph data structures

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;

/// Unique identifier for a module
pub type ModuleId = usize;

/// Types of modules the bundler can handle
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModuleType {
    JavaScript,
    TypeScript,
    Jsx,
    Tsx,
    Css,
    Json,
    Unknown,
}

impl ModuleType {
    /// Determine module type from file extension
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "js" | "mjs" | "cjs" => ModuleType::JavaScript,
            "ts" | "mts" | "cts" => ModuleType::TypeScript,
            "jsx" => ModuleType::Jsx,
            "tsx" => ModuleType::Tsx,
            "css" | "scss" | "sass" | "less" => ModuleType::Css,
            "json" => ModuleType::Json,
            _ => ModuleType::Unknown,
        }
    }
    
    /// Check if this is a JavaScript-like module
    pub fn is_js_like(&self) -> bool {
        matches!(
            self,
            ModuleType::JavaScript
                | ModuleType::TypeScript
                | ModuleType::Jsx
                | ModuleType::Tsx
        )
    }
}

/// A module in the dependency graph
#[derive(Debug, Clone)]
pub struct Module {
    /// Absolute path to the module
    pub path: PathBuf,
    
    /// Original source code
    pub source: String,
    
    /// Module type
    pub module_type: ModuleType,
    
    /// Whether this is an entry point
    pub is_entry: bool,
    
    /// Import specifiers found in this module
    pub dependencies: Vec<String>,
    
    /// Transformed code (after TypeScript/JSX compilation)
    pub transformed: Option<String>,
}

impl Module {
    /// Detect module type from path
    pub fn detect_type(path: &PathBuf) -> ModuleType {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(ModuleType::from_extension)
            .unwrap_or(ModuleType::Unknown)
    }
}

/// The module dependency graph
#[derive(Debug, Default)]
pub struct ModuleGraph {
    /// All modules indexed by their ID
    modules: HashMap<ModuleId, Module>,
    
    /// Map from path to module ID
    path_to_id: HashMap<PathBuf, ModuleId>,
    
    /// Dependency edges: module ID -> set of dependency IDs
    edges: HashMap<ModuleId, HashSet<ModuleId>>,
    
    /// Next available module ID
    next_id: ModuleId,
}

impl ModuleGraph {
    /// Create a new empty module graph
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Add a module to the graph
    pub fn add_module(&mut self, module: Module) -> ModuleId {
        let path = module.path.clone();
        
        // Check if already exists
        if let Some(&id) = self.path_to_id.get(&path) {
            return id;
        }
        
        let id = self.next_id;
        self.next_id += 1;
        
        self.path_to_id.insert(path, id);
        self.modules.insert(id, module);
        self.edges.insert(id, HashSet::new());
        
        id
    }
    
    /// Add a dependency edge between modules
    pub fn add_dependency(&mut self, from: ModuleId, to: ModuleId) {
        if let Some(deps) = self.edges.get_mut(&from) {
            deps.insert(to);
        }
    }
    
    /// Get module ID from path
    pub fn get_module_id(&self, path: &PathBuf) -> Option<ModuleId> {
        self.path_to_id.get(path).copied()
    }
    
    /// Get a module by ID
    pub fn get_module(&self, id: ModuleId) -> Option<&Module> {
        self.modules.get(&id)
    }
    
    /// Get a mutable reference to a module
    pub fn get_module_mut(&mut self, id: ModuleId) -> Option<&mut Module> {
        self.modules.get_mut(&id)
    }
    
    /// Get all module IDs
    pub fn all_module_ids(&self) -> Vec<ModuleId> {
        self.modules.keys().copied().collect()
    }
    
    /// Get all modules reachable from a given module (BFS)
    pub fn get_reachable_modules(&self, start: ModuleId) -> Vec<ModuleId> {
        let mut visited = HashSet::new();
        let mut result = Vec::new();
        let mut queue = VecDeque::new();
        
        queue.push_back(start);
        visited.insert(start);
        
        while let Some(id) = queue.pop_front() {
            result.push(id);
            
            if let Some(deps) = self.edges.get(&id) {
                for &dep_id in deps {
                    if visited.insert(dep_id) {
                        queue.push_back(dep_id);
                    }
                }
            }
        }
        
        result
    }
    
    /// Get direct dependencies of a module
    pub fn get_dependencies(&self, id: ModuleId) -> Vec<ModuleId> {
        self.edges
            .get(&id)
            .map(|deps| deps.iter().copied().collect())
            .unwrap_or_default()
    }
    
    /// Get entry point modules
    pub fn get_entry_modules(&self) -> Vec<ModuleId> {
        self.modules
            .iter()
            .filter(|(_, m)| m.is_entry)
            .map(|(&id, _)| id)
            .collect()
    }
    
    /// Total number of modules
    pub fn len(&self) -> usize {
        self.modules.len()
    }
    
    /// Check if graph is empty
    pub fn is_empty(&self) -> bool {
        self.modules.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_module_type_detection() {
        assert_eq!(ModuleType::from_extension("js"), ModuleType::JavaScript);
        assert_eq!(ModuleType::from_extension("ts"), ModuleType::TypeScript);
        assert_eq!(ModuleType::from_extension("jsx"), ModuleType::Jsx);
        assert_eq!(ModuleType::from_extension("tsx"), ModuleType::Tsx);
        assert_eq!(ModuleType::from_extension("css"), ModuleType::Css);
        assert_eq!(ModuleType::from_extension("json"), ModuleType::Json);
        assert_eq!(ModuleType::from_extension("xyz"), ModuleType::Unknown);
    }
    
    #[test]
    fn test_module_graph_basic() {
        let mut graph = ModuleGraph::new();
        
        let module = Module {
            path: PathBuf::from("/test/main.js"),
            source: "console.log('test')".to_string(),
            module_type: ModuleType::JavaScript,
            is_entry: true,
            dependencies: vec![],
            transformed: None,
        };
        
        let id = graph.add_module(module);
        assert_eq!(graph.len(), 1);
        assert!(graph.get_module(id).is_some());
        assert_eq!(graph.get_module_id(&PathBuf::from("/test/main.js")), Some(id));
    }
}
