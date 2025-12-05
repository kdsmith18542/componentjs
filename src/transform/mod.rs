//! Code transformation
//!
//! Handles TypeScript, JSX, and other transformations using SWC.

use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};
use tracing::debug;

use crate::bundler::ModuleType;
use crate::config::Config;

/// Code transformer using SWC
pub struct Transformer {
    /// Project configuration
    config: Arc<Config>,
}

impl Transformer {
    /// Create a new transformer
    pub fn new(config: Arc<Config>) -> Result<Self> {
        Ok(Self { config })
    }
    
    /// Transform source code based on module type
    pub fn transform(
        &self,
        source: &str,
        path: &Path,
        module_type: &ModuleType,
    ) -> Result<String> {
        match module_type {
            ModuleType::TypeScript => self.transform_typescript(source, path),
            ModuleType::Tsx => self.transform_tsx(source, path),
            ModuleType::Jsx => self.transform_jsx(source, path),
            ModuleType::Css => self.transform_css(source, path),
            ModuleType::Json => self.transform_json(source, path),
            _ => Ok(source.to_string()),
        }
    }
    
    /// Transform TypeScript to JavaScript
    fn transform_typescript(&self, source: &str, path: &Path) -> Result<String> {
        debug!("Transforming TypeScript: {}", path.display());
        
        // For now, we'll do a simple transformation that removes type annotations
        // In a full implementation, we'd use swc_ecma_parser and swc_ecma_transforms_typescript
        
        let result = self.strip_typescript_types(source)?;
        
        Ok(result)
    }
    
    /// Transform TSX to JavaScript
    fn transform_tsx(&self, source: &str, path: &Path) -> Result<String> {
        debug!("Transforming TSX: {}", path.display());
        
        // First strip TypeScript types, then transform JSX
        let without_types = self.strip_typescript_types(source)?;
        self.transform_jsx(&without_types, path)
    }
    
    /// Transform JSX to JavaScript
    fn transform_jsx(&self, source: &str, path: &Path) -> Result<String> {
        debug!("Transforming JSX: {}", path.display());
        
        if !self.config.features.jsx {
            return Ok(source.to_string());
        }
        
        // Simple JSX transformation
        // In a full implementation, we'd use swc_ecma_transforms_react
        let result = self.transform_jsx_simple(source)?;
        
        Ok(result)
    }
    
    /// Transform CSS (wrap as JS module)
    fn transform_css(&self, source: &str, path: &Path) -> Result<String> {
        debug!("Transforming CSS: {}", path.display());
        
        // Wrap CSS as a JS module that injects styles
        let escaped = source
            .replace('\\', "\\\\")
            .replace('`', "\\`")
            .replace("${", "\\${");
        
        let js_module = format!(
            r#"(function() {{
  var style = document.createElement('style');
  style.textContent = `{}`;
  document.head.appendChild(style);
}})();
module.exports = {{}};
"#,
            escaped
        );
        
        Ok(js_module)
    }
    
    /// Transform JSON to JS module
    fn transform_json(&self, source: &str, path: &Path) -> Result<String> {
        debug!("Transforming JSON: {}", path.display());
        
        // Validate JSON
        serde_json::from_str::<serde_json::Value>(source)
            .with_context(|| format!("Invalid JSON in {}", path.display()))?;
        
        Ok(format!("module.exports = {};", source))
    }
    
    /// Simple TypeScript type stripping
    /// This is a basic implementation - a full solution would use SWC's TypeScript transforms
    fn strip_typescript_types(&self, source: &str) -> Result<String> {
        let mut result = String::with_capacity(source.len());
        let mut chars = source.chars().peekable();
        let mut in_string = false;
        let mut string_char = '"';
        let mut in_template = false;
        let mut template_depth: i32 = 0;
        
        while let Some(c) = chars.next() {
            // Handle string literals
            if !in_template && (c == '"' || c == '\'') {
                if !in_string {
                    in_string = true;
                    string_char = c;
                } else if string_char == c {
                    in_string = false;
                }
                result.push(c);
                continue;
            }
            
            // Handle template literals
            if c == '`' {
                if !in_template {
                    in_template = true;
                    template_depth = 0;
                } else if template_depth == 0 {
                    in_template = false;
                }
                result.push(c);
                continue;
            }
            
            // Inside strings/templates, just copy
            if in_string || in_template {
                if in_template && c == '{' {
                    template_depth += 1;
                } else if in_template && c == '}' {
                    template_depth = template_depth.saturating_sub(1);
                }
                result.push(c);
                continue;
            }
            
            // Skip type annotations after colons (simplified)
            if c == ':' {
                // Check if this might be a type annotation
                // Look ahead for common patterns
                let rest: String = chars.clone().take(50).collect();
                
                // Skip if followed by type-like patterns
                if rest.starts_with(' ') {
                    let trimmed = rest.trim_start();
                    // Check for common type patterns
                    if trimmed.starts_with("string")
                        || trimmed.starts_with("number")
                        || trimmed.starts_with("boolean")
                        || trimmed.starts_with("any")
                        || trimmed.starts_with("void")
                        || trimmed.starts_with("never")
                        || trimmed.starts_with("unknown")
                        || trimmed.starts_with("null")
                        || trimmed.starts_with("undefined")
                        || trimmed.starts_with("Array<")
                        || trimmed.starts_with("Promise<")
                        || trimmed.starts_with("Record<")
                        || trimmed.starts_with("Map<")
                        || trimmed.starts_with("Set<")
                        || trimmed.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)
                    {
                        // Skip until we hit =, ), ,, {, or newline
                        let mut depth = 0;
                        while let Some(&next) = chars.peek() {
                            if next == '<' || next == '(' || next == '[' {
                                depth += 1;
                                chars.next();
                            } else if next == '>' || next == ')' || next == ']' {
                                depth -= 1;
                                chars.next();
                            } else if depth == 0 && (next == '=' || next == ')' || next == ',' || next == '{' || next == ';' || next == '\n') {
                                break;
                            } else {
                                chars.next();
                            }
                        }
                        continue;
                    }
                }
            }
            
            // Skip interface/type declarations
            if c == 'i' {
                let rest: String = std::iter::once(c).chain(chars.clone().take(10)).collect();
                if rest.starts_with("interface ") {
                    // Skip until opening brace, then skip the whole block
                    while let Some(nc) = chars.next() {
                        if nc == '{' {
                            let mut depth = 1;
                            while depth > 0 {
                                if let Some(bc) = chars.next() {
                                    if bc == '{' { depth += 1; }
                                    else if bc == '}' { depth -= 1; }
                                }
                            }
                            break;
                        }
                    }
                    continue;
                }
            }
            
            if c == 't' {
                let rest: String = std::iter::once(c).chain(chars.clone().take(5)).collect();
                if rest.starts_with("type ") {
                    // Skip until semicolon or newline
                    while let Some(nc) = chars.next() {
                        if nc == ';' || nc == '\n' {
                            break;
                        }
                    }
                    continue;
                }
            }
            
            // Remove 'as Type' casts (simplified)
            if c == ' ' {
                let rest: String = chars.clone().take(3).collect();
                if rest == "as " {
                    // Skip "as Type"
                    for _ in 0..3 {
                        chars.next();
                    }
                    // Skip the type name
                    let mut depth = 0;
                    while let Some(&next) = chars.peek() {
                        if next == '<' || next == '(' {
                            depth += 1;
                            chars.next();
                        } else if next == '>' || next == ')' {
                            depth -= 1;
                            chars.next();
                        } else if depth == 0 && (next.is_whitespace() || next == ',' || next == ')' || next == ';' || next == '}') {
                            break;
                        } else {
                            chars.next();
                        }
                    }
                    result.push(' ');
                    continue;
                }
            }
            
            result.push(c);
        }
        
        Ok(result)
    }
    
    /// Simple JSX transformation
    /// Transforms JSX syntax to React.createElement calls
    fn transform_jsx_simple(&self, source: &str) -> Result<String> {
        let mut result = String::with_capacity(source.len());
        let mut chars = source.chars().peekable();
        let mut in_string = false;
        let mut string_char = '"';
        
        while let Some(c) = chars.next() {
            // Handle strings
            if c == '"' || c == '\'' || c == '`' {
                if !in_string {
                    in_string = true;
                    string_char = c;
                } else if string_char == c {
                    in_string = false;
                }
                result.push(c);
                continue;
            }
            
            if in_string {
                result.push(c);
                continue;
            }
            
            // Check for JSX
            if c == '<' {
                // Check if this looks like JSX
                if let Some(&next) = chars.peek() {
                    if next.is_alphabetic() || next == '>' || next == '/' {
                        // Try to parse JSX element
                        let jsx_result = self.parse_jsx_element(&mut chars, c)?;
                        result.push_str(&jsx_result);
                        continue;
                    }
                }
            }
            
            result.push(c);
        }
        
        Ok(result)
    }
    
    /// Parse a single JSX element
    fn parse_jsx_element(&self, chars: &mut std::iter::Peekable<std::str::Chars>, start: char) -> Result<String> {
        let mut element = String::from(start);
        let mut depth = 1;
        
        // Collect the full JSX element
        while depth > 0 {
            if let Some(c) = chars.next() {
                element.push(c);
                if c == '<' {
                    if chars.peek() != Some(&'/') {
                        depth += 1;
                    }
                } else if c == '>' {
                    // Check if previous char was /
                    if element.len() >= 2 && element.chars().nth(element.len() - 2) == Some('/') {
                        depth -= 1;
                    } else if element.contains("</") {
                        depth -= 1;
                    }
                } else if c == '/' && chars.peek() == Some(&'>') {
                    // Self-closing tag
                    element.push(chars.next().unwrap());
                    depth -= 1;
                }
            } else {
                break;
            }
        }
        
        // For now, just wrap in a comment with the original
        // A full implementation would properly transform to createElement calls
        let _import_source = &self.config.features.jsx_import_source;
        
        // Very basic transformation for simple cases
        if element.starts_with("<>") {
            return Ok(format!("React.createElement(React.Fragment, null)"));
        }
        
        // Return original for complex cases (proper parsing needed)
        Ok(element)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    
    #[test]
    fn test_transform_json() {
        let config = Config::default_config();
        let transformer = Transformer::new(Arc::new(config)).unwrap();
        
        let json = r#"{"key": "value", "num": 42}"#;
        let result = transformer.transform_json(json, Path::new("test.json")).unwrap();
        
        assert!(result.starts_with("module.exports = "));
    }
    
    #[test]
    fn test_transform_css() {
        let config = Config::default_config();
        let transformer = Transformer::new(Arc::new(config)).unwrap();
        
        let css = "body { color: red; }";
        let result = transformer.transform_css(css, Path::new("test.css")).unwrap();
        
        assert!(result.contains("document.createElement('style')"));
        assert!(result.contains("body { color: red; }"));
    }
}
