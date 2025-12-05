//! Chunk generation for code splitting

use super::ModuleId;

/// Type of chunk
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChunkType {
    /// Entry point chunk - loaded immediately
    Entry,
    /// Async chunk - loaded on demand via dynamic import
    Async,
    /// Shared chunk - contains modules used by multiple entry points
    Shared,
}

/// A chunk is a group of modules that will be bundled together
#[derive(Debug, Clone)]
pub struct Chunk {
    /// Chunk name (used for output filename)
    pub name: String,
    
    /// Type of chunk
    pub chunk_type: ChunkType,
    
    /// Module IDs included in this chunk
    pub module_ids: Vec<ModuleId>,
}

impl Chunk {
    /// Create a new entry chunk
    pub fn entry(name: String, module_ids: Vec<ModuleId>) -> Self {
        Self {
            name,
            chunk_type: ChunkType::Entry,
            module_ids,
        }
    }
    
    /// Create a new async chunk
    pub fn async_chunk(name: String, module_ids: Vec<ModuleId>) -> Self {
        Self {
            name,
            chunk_type: ChunkType::Async,
            module_ids,
        }
    }
    
    /// Create a new shared chunk
    pub fn shared(name: String, module_ids: Vec<ModuleId>) -> Self {
        Self {
            name,
            chunk_type: ChunkType::Shared,
            module_ids,
        }
    }
    
    /// Check if chunk is empty
    pub fn is_empty(&self) -> bool {
        self.module_ids.is_empty()
    }
    
    /// Number of modules in chunk
    pub fn len(&self) -> usize {
        self.module_ids.len()
    }
}
