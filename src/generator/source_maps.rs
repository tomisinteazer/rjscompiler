//! # Source Maps Component (Component 13)
//!
//! Implements Source Maps V3 specification for mapping generated code back to original sources.
//! Supports external, inline, and indexed source maps with token-level or statement-level granularity.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Source Map V3 structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMap {
    /// Source map version (always 3)
    pub version: u8,
    /// Output file name (optional but recommended)
    pub file: Option<String>,
    /// Source root path (optional)
    #[serde(rename = "sourceRoot", skip_serializing_if = "Option::is_none")]
    pub source_root: Option<String>,
    /// List of source file paths
    pub sources: Vec<String>,
    /// Optional inline source content
    #[serde(rename = "sourcesContent", skip_serializing_if = "Option::is_none")]
    pub sources_content: Option<Vec<String>>,
    /// List of symbol names
    pub names: Vec<String>,
    /// Base64 VLQ encoded mappings
    pub mappings: String,
    /// Optional sections for indexed source maps
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sections: Option<Vec<SourceMapSection>>,
}

/// Source map section for indexed maps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMapSection {
    /// Offset in generated file
    pub offset: SourceMapOffset,
    /// Map for this section
    pub map: SourceMap,
}

/// Offset for source map sections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMapOffset {
    /// Line offset
    pub line: u32,
    /// Column offset
    pub column: u32,
}

/// Mapping segment before VLQ encoding
#[derive(Debug, Clone, PartialEq)]
pub struct MappingSegment {
    /// Generated column (0-based)
    pub generated_column: u32,
    /// Source file index (optional)
    pub source_index: Option<u32>,
    /// Original line (0-based, optional)
    pub original_line: Option<u32>,
    /// Original column (0-based, optional)
    pub original_column: Option<u32>,
    /// Name index (optional)
    pub name_index: Option<u32>,
}

/// Position in source or generated code
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    /// Line number (0-based)
    pub line: u32,
    /// Column number (0-based)
    pub column: u32,
}

/// Mapping from generated position to original position
#[derive(Debug, Clone)]
pub struct Mapping {
    /// Position in generated code
    pub generated: Position,
    /// Position in original code (optional)
    pub original: Option<Position>,
    /// Source file index (optional)
    pub source_index: Option<u32>,
    /// Symbol name index (optional)
    pub name_index: Option<u32>,
}

/// Source map builder for constructing source maps during generation
pub struct SourceMapBuilder {
    /// Source file paths
    sources: Vec<String>,
    /// Source content (optional)
    sources_content: Option<Vec<String>>,
    /// Symbol names
    names: Vec<String>,
    /// Mappings grouped by generated line
    mappings: HashMap<u32, Vec<MappingSegment>>,
    /// Current generated position
    generated_line: u32,
    /// Source file lookup
    source_lookup: HashMap<String, u32>,
    /// Name lookup
    name_lookup: HashMap<String, u32>,
}

impl SourceMap {
    /// Create a new empty source map
    pub fn new() -> Self {
        Self {
            version: 3,
            file: None,
            source_root: None,
            sources: Vec::new(),
            sources_content: None,
            names: Vec::new(),
            mappings: String::new(),
            sections: None,
        }
    }

    /// Create source map from builder
    pub fn from_builder(builder: SourceMapBuilder) -> Self {
        let mappings = encode_mappings(&builder.mappings);
        
        Self {
            version: 3,
            file: None,
            source_root: None,
            sources: builder.sources,
            sources_content: builder.sources_content,
            names: builder.names,
            mappings,
            sections: None,
        }
    }

    /// Convert to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Convert to inline data URL
    pub fn to_inline_data_url(&self) -> Result<String, serde_json::Error> {
        let json = self.to_json()?;
        let encoded = base64::encode(&json);
        Ok(format!("data:application/json;charset=utf-8;base64,{}", encoded))
    }

    /// Add source mapping URL comment
    pub fn add_source_mapping_url_comment(&self, url: &str) -> String {
        format!("//# sourceMappingURL={}", url)
    }
}

impl Default for SourceMap {
    fn default() -> Self {
        Self::new()
    }
}

impl SourceMapBuilder {
    /// Create a new source map builder
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
            sources_content: None,
            names: Vec::new(),
            mappings: HashMap::new(),
            generated_line: 0,
            source_lookup: HashMap::new(),
            name_lookup: HashMap::new(),
        }
    }

    /// Add a source file
    pub fn add_source(&mut self, source_path: &str) -> u32 {
        if let Some(&index) = self.source_lookup.get(source_path) {
            return index;
        }

        let index = self.sources.len() as u32;
        self.sources.push(source_path.to_string());
        self.source_lookup.insert(source_path.to_string(), index);
        index
    }

    /// Add source content
    pub fn add_source_content(&mut self, content: &str) {
        if self.sources_content.is_none() {
            self.sources_content = Some(Vec::new());
        }
        
        if let Some(ref mut contents) = self.sources_content {
            contents.push(content.to_string());
        }
    }

    /// Add a symbol name
    pub fn add_name(&mut self, name: &str) -> u32 {
        if let Some(&index) = self.name_lookup.get(name) {
            return index;
        }

        let index = self.names.len() as u32;
        self.names.push(name.to_string());
        self.name_lookup.insert(name.to_string(), index);
        index
    }

    /// Add a mapping
    pub fn add_mapping(&mut self, mapping: Mapping) {
        let segment = MappingSegment {
            generated_column: mapping.generated.column,
            source_index: mapping.source_index,
            original_line: mapping.original.map(|p| p.line),
            original_column: mapping.original.map(|p| p.column),
            name_index: mapping.name_index,
        };

        self.mappings
            .entry(mapping.generated.line)
            .or_insert_with(Vec::new)
            .push(segment);
    }

    /// Set current generated line (for newline tracking)
    pub fn set_generated_line(&mut self, line: u32) {
        self.generated_line = line;
    }

    /// Build the final source map
    pub fn build(self) -> SourceMap {
        SourceMap::from_builder(self)
    }
}

impl Default for SourceMapBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Encode mappings to Base64 VLQ format
fn encode_mappings(mappings: &HashMap<u32, Vec<MappingSegment>>) -> String {
    let mut result = String::new();
    let mut prev_generated_column = 0;
    let mut prev_source_index = 0;
    let mut prev_original_line = 0;
    let mut prev_original_column = 0;
    let mut prev_name_index = 0;

    // Process mappings in line order
    let mut sorted_lines: Vec<_> = mappings.keys().collect();
    sorted_lines.sort();

    for (line_idx, &line) in sorted_lines.iter().enumerate() {
        if line_idx > 0 {
            result.push(';');
        }

        // Reset column for new line
        prev_generated_column = 0;

        if let Some(segments) = mappings.get(line) {
            let mut sorted_segments = segments.clone();
            sorted_segments.sort_by_key(|s| s.generated_column);

            for (seg_idx, segment) in sorted_segments.iter().enumerate() {
                if seg_idx > 0 {
                    result.push(',');
                }

                // Generated column (always present)
                result.push_str(&encode_vlq(segment.generated_column as i32 - prev_generated_column as i32));
                prev_generated_column = segment.generated_column;

                // Source information (optional)
                if let (Some(source_idx), Some(orig_line), Some(orig_col)) = 
                    (segment.source_index, segment.original_line, segment.original_column) {
                    
                    // Source index
                    result.push_str(&encode_vlq(source_idx as i32 - prev_source_index as i32));
                    prev_source_index = source_idx;

                    // Original line
                    result.push_str(&encode_vlq(orig_line as i32 - prev_original_line as i32));
                    prev_original_line = orig_line;

                    // Original column
                    result.push_str(&encode_vlq(orig_col as i32 - prev_original_column as i32));
                    prev_original_column = orig_col;

                    // Name index (optional)
                    if let Some(name_idx) = segment.name_index {
                        result.push_str(&encode_vlq(name_idx as i32 - prev_name_index as i32));
                        prev_name_index = name_idx;
                    }
                }
            }
        }
    }

    result
}

/// Encode a signed integer as Base64 VLQ
fn encode_vlq(mut value: i32) -> String {
    let mut result = String::new();
    let sign = if value < 0 { 1 } else { 0 };
    value = value.abs();

    // Encode sign in LSB
    let mut vlq = (value << 1) | sign;

    loop {
        let mut digit = vlq & 0x1f;
        vlq >>= 5;

        if vlq != 0 {
            digit |= 0x20; // Continuation bit
        }

        result.push(encode_base64_digit(digit as u8));

        if vlq == 0 {
            break;
        }
    }

    result
}

/// Encode a 6-bit value as Base64 character
fn encode_base64_digit(value: u8) -> char {
    match value {
        0..=25 => (b'A' + value) as char,
        26..=51 => (b'a' + (value - 26)) as char,
        52..=61 => (b'0' + (value - 52)) as char,
        62 => '+',
        63 => '/',
        _ => panic!("Invalid base64 digit: {}", value),
    }
}

// Note: This is a simplified base64 implementation. In a production system,
// you might want to use the `base64` crate instead.
mod base64 {
    pub fn encode(data: &str) -> String {
        // Simplified base64 encoding
        let mut result = String::new();
        let bytes = data.as_bytes();
        
        for chunk in bytes.chunks(3) {
            let mut buffer = [0u8; 3];
            for (i, &byte) in chunk.iter().enumerate() {
                buffer[i] = byte;
            }
            
            let combined = ((buffer[0] as u32) << 16) | 
                          ((buffer[1] as u32) << 8) | 
                          (buffer[2] as u32);
            
            result.push(encode_char((combined >> 18) & 0x3f));
            result.push(encode_char((combined >> 12) & 0x3f));
            
            if chunk.len() > 1 {
                result.push(encode_char((combined >> 6) & 0x3f));
            } else {
                result.push('=');
            }
            
            if chunk.len() > 2 {
                result.push(encode_char(combined & 0x3f));
            } else {
                result.push('=');
            }
        }
        
        result
    }
    
    fn encode_char(value: u32) -> char {
        match value {
            0..=25 => (b'A' + value as u8) as char,
            26..=51 => (b'a' + (value as u8 - 26)) as char,
            52..=61 => (b'0' + (value as u8 - 52)) as char,
            62 => '+',
            63 => '/',
            _ => panic!("Invalid base64 value: {}", value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_map_creation() {
        let map = SourceMap::new();
        assert_eq!(map.version, 3);
        assert!(map.sources.is_empty());
        assert!(map.names.is_empty());
    }

    #[test]
    fn test_source_map_builder() {
        let mut builder = SourceMapBuilder::new();
        let source_index = builder.add_source("test.js");
        let name_index = builder.add_name("test");
        
        assert_eq!(source_index, 0);
        assert_eq!(name_index, 0);
        assert_eq!(builder.sources.len(), 1);
        assert_eq!(builder.names.len(), 1);
    }

    #[test]
    fn test_mapping_addition() {
        let mut builder = SourceMapBuilder::new();
        let source_index = builder.add_source("test.js");
        
        let mapping = Mapping {
            generated: Position { line: 0, column: 0 },
            original: Some(Position { line: 0, column: 0 }),
            source_index: Some(source_index),
            name_index: None,
        };
        
        builder.add_mapping(mapping);
        assert!(builder.mappings.contains_key(&0));
    }

    #[test]
    fn test_vlq_encoding() {
        assert_eq!(encode_vlq(0), "A");
        assert_eq!(encode_vlq(1), "C");
        assert_eq!(encode_vlq(-1), "D");
    }

    #[test]
    fn test_base64_digit_encoding() {
        assert_eq!(encode_base64_digit(0), 'A');
        assert_eq!(encode_base64_digit(25), 'Z');
        assert_eq!(encode_base64_digit(26), 'a');
        assert_eq!(encode_base64_digit(51), 'z');
        assert_eq!(encode_base64_digit(52), '0');
        assert_eq!(encode_base64_digit(61), '9');
        assert_eq!(encode_base64_digit(62), '+');
        assert_eq!(encode_base64_digit(63), '/');
    }
}