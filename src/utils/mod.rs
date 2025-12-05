//! Utility functions and helpers

use std::path::Path;

use sha2::{Digest, Sha256};

/// Generate a hash of the given content
pub fn hash_content(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    let result = hasher.finalize();
    hex::encode(&result[..8])
}

/// Generate a hash-based filename
pub fn hash_filename(base: &str, content: &[u8], ext: &str) -> String {
    let hash = hash_content(content);
    format!("{}.{}.{}", base, hash, ext)
}

/// Check if a path is within a directory
pub fn is_subpath(path: &Path, base: &Path) -> bool {
    path.canonicalize()
        .ok()
        .and_then(|p| {
            base.canonicalize()
                .ok()
                .map(|b| p.starts_with(&b))
        })
        .unwrap_or(false)
}

/// Get relative path from base to target
pub fn relative_path(from: &Path, to: &Path) -> Option<String> {
    pathdiff::diff_paths(to, from)
        .map(|p| p.display().to_string())
}

/// Clean a path by removing . and .. components
pub fn clean_path(path: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();
    
    for part in path.split('/') {
        match part {
            "" | "." => continue,
            ".." => {
                parts.pop();
            }
            _ => parts.push(part),
        }
    }
    
    if path.starts_with('/') {
        format!("/{}", parts.join("/"))
    } else {
        parts.join("/")
    }
}

/// Convert a file path to a module ID
pub fn path_to_module_id(path: &Path) -> String {
    path.display()
        .to_string()
        .replace('\\', "/")
}

/// Format bytes as human-readable size
pub fn format_size(bytes: usize) -> String {
    const KB: usize = 1024;
    const MB: usize = KB * 1024;
    const GB: usize = MB * 1024;
    
    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Format duration as human-readable string
pub fn format_duration(duration: std::time::Duration) -> String {
    let secs = duration.as_secs_f64();
    
    if secs >= 60.0 {
        let mins = (secs / 60.0).floor() as u64;
        let remaining_secs = secs - (mins as f64 * 60.0);
        format!("{}m {:.2}s", mins, remaining_secs)
    } else if secs >= 1.0 {
        format!("{:.2}s", secs)
    } else {
        format!("{:.0}ms", secs * 1000.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hash_content() {
        let hash = hash_content(b"hello world");
        assert_eq!(hash.len(), 16);
    }
    
    #[test]
    fn test_clean_path() {
        assert_eq!(clean_path("./foo/bar"), "foo/bar");
        assert_eq!(clean_path("foo/../bar"), "bar");
        assert_eq!(clean_path("/foo/./bar/../baz"), "/foo/baz");
    }
    
    #[test]
    fn test_format_size() {
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1536), "1.50 KB");
        assert_eq!(format_size(1048576), "1.00 MB");
    }
    
    #[test]
    fn test_format_duration() {
        use std::time::Duration;
        
        assert_eq!(format_duration(Duration::from_millis(500)), "500ms");
        assert_eq!(format_duration(Duration::from_secs_f64(1.5)), "1.50s");
        assert_eq!(format_duration(Duration::from_secs(65)), "1m 5.00s");
    }
}
