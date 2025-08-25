//! # Code Generator Module
//!
//! The generator component converts the transformed/annotated AST into minimal, correct JavaScript
//! and (optionally) Source Maps v3, with deterministic output and strict safety for ASI,
//! precedence, and encoding.
//!
//! ## Overview
//!
//! This module implements Phase 5 of the JavaScript minifier pipeline:
//! - **Input**: Transformed AST from Phase 4 with scope info, rename map, and semantic flags
//! - **Output**: Minified JavaScript code with optional source maps
//! - **Approach**: TDD (Test Driven Development) following the comprehensive specification
//!
//! ## Components
//!
//! - **Printer**: Component 12 - AST traversal and token emission with minimal bytes
//! - **Source Maps**: Component 13 - Source Maps V3 generation with position tracking
//! - **Configuration**: Output formatting, optimization levels, and source map options
//!
//! ## Safety Guarantees
//!
//! - **Semantic Preservation**: Maintains exact JavaScript runtime behavior
//! - **ASI Safety**: Proper handling of Automatic Semicolon Insertion hazards
//! - **Precedence Correctness**: Accurate operator precedence and associativity
//! - **Unicode Safety**: Proper handling of all Unicode characters and escapes

use crate::parser::ast_types::*;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod printer;
pub mod source_maps;

#[cfg(test)]
mod tests;

/// Generator configuration for output formatting and optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratorConfig {
    /// ECMAScript target version
    pub ecma: EcmaScriptVersion,
    /// Output format style
    pub format: OutputFormat,
    /// Semicolon insertion strategy
    pub semicolon: SemicolonStrategy,
    /// Quote character preference
    pub quote: QuoteStrategy,
    /// Comment preservation level
    pub preserve_comments: CommentPreservation,
    /// Source map generation mode
    pub source_map: SourceMapMode,
    /// Source root for source maps
    pub source_root: Option<String>,
    /// Include sources content in source maps
    pub include_sources_content: bool,
    /// Mapping granularity level
    pub mapping_granularity: MappingGranularity,
    /// Output newline style
    pub newline: NewlineStyle,
    /// Maximum line length for wrapping
    pub max_line_len: Option<usize>,
    /// Character set escape mode
    pub charset_escapes: CharsetEscapes,
}

/// ECMAScript version target
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EcmaScriptVersion {
    ES5,
    ES2015,
    Latest,
}

/// Output format style
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum OutputFormat {
    /// Compact minified output
    Compact,
    /// Readable with some formatting
    Readable,
    /// Pretty printed with full formatting
    Pretty,
}

/// Semicolon insertion strategy
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SemicolonStrategy {
    /// Automatic insertion based on ASI rules
    Auto,
    /// Always insert semicolons
    Always,
    /// Remove unnecessary semicolons
    Remove,
}

/// Quote character strategy
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum QuoteStrategy {
    /// Automatically choose based on content
    Auto,
    /// Prefer single quotes
    Single,
    /// Prefer double quotes
    Double,
}

/// Comment preservation level
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CommentPreservation {
    /// Remove all comments
    None,
    /// Preserve license comments only
    License,
    /// Preserve all comments
    All,
}

/// Source map generation mode
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SourceMapMode {
    /// No source map generation
    None,
    /// External source map file
    File,
    /// Inline source map as data URL
    Inline,
    /// Indexed source map for multiple files
    Indexed,
}

/// Mapping granularity for source maps
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MappingGranularity {
    /// Token-level mapping (default)
    Token,
    /// Statement-level mapping (smaller maps)
    Statement,
}

/// Newline style for output
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum NewlineStyle {
    /// Unix-style LF
    Lf,
    /// Windows-style CRLF
    Crlf,
}

/// Character set escape mode
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CharsetEscapes {
    /// Minimal escaping
    Minimal,
    /// ASCII-only output with escapes
    AsciiOnly,
}

/// Generator result containing generated code and optional source map
#[derive(Debug, Clone)]
pub struct GenerationResult {
    /// Generated JavaScript code
    pub code: String,
    /// Optional source map
    pub source_map: Option<source_maps::SourceMap>,
    /// Generation diagnostics and metrics
    pub diagnostics: GeneratorDiagnostics,
}

/// Generation diagnostics and metrics
#[derive(Debug, Clone)]
pub struct GeneratorDiagnostics {
    /// Original code size in bytes
    pub original_size: usize,
    /// Generated code size in bytes
    pub generated_size: usize,
    /// Compression ratio (0.0 to 1.0)
    pub compression_ratio: f64,
    /// Generation time in milliseconds
    pub generation_time_ms: f64,
    /// Number of warnings generated
    pub warning_count: usize,
    /// Specific warnings
    pub warnings: Vec<String>,
}

/// Generator error types
#[derive(Debug, Error)]
pub enum GeneratorError {
    #[error("Malformed AST: {message} at node type {node_type}")]
    MalformedAst {
        message: String,
        node_type: String,
    },
    #[error("Unsupported node type for target {target}: {node_type}")]
    UnsupportedNode {
        target: String,
        node_type: String,
    },
    #[error("Source map generation failed: {message}")]
    SourceMapError { message: String },
    #[error("Path security violation: {path}")]
    PathSecurityError { path: String },
    #[error("IO error during generation: {0}")]
    IoError(#[from] std::io::Error),
    #[error("JSON serialization error: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Invalid AST structure: {field} is required but missing in {node_type}")]
    MissingRequiredField {
        field: String,
        node_type: String,
    },
    #[error("Invalid expression precedence: cannot determine precedence for {operator} in context {context}")]
    InvalidPrecedence {
        operator: String,
        context: String,
    },
    #[error("String processing error: {message} in string: {content}")]
    StringProcessingError {
        message: String,
        content: String,
    },
    #[error("Numeric value error: {message} for value: {value}")]
    NumericValueError {
        message: String,
        value: String,
    },
    #[error("ASI hazard detected: {message} at position {line}:{column}")]
    AsiHazard {
        message: String,
        line: u32,
        column: u32,
    },
    #[error("Memory limit exceeded: {current_usage} bytes exceeds limit of {limit} bytes")]
    MemoryLimitExceeded {
        current_usage: usize,
        limit: usize,
    },
    #[error("Output size limit exceeded: {current_size} bytes exceeds limit of {limit} bytes")]
    OutputSizeLimitExceeded {
        current_size: usize,
        limit: usize,
    },
    #[error("Generation timeout: operation exceeded {timeout_ms}ms")]
    GenerationTimeout { timeout_ms: u64 },
    #[error("Invalid configuration: {message}")]
    InvalidConfiguration { message: String },
    #[error("Template literal error: {message} in template: {template}")]
    TemplateLiteralError {
        message: String,
        template: String,
    },
    #[error("Identifier error: {message} for identifier: {identifier}")]
    IdentifierError {
        message: String,
        identifier: String,
    },
}

/// Result type alias for generator operations
pub type GeneratorResult<T> = Result<T, GeneratorError>;

/// Main generator implementation
pub struct Generator {
    config: GeneratorConfig,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            ecma: EcmaScriptVersion::Latest,
            format: OutputFormat::Compact,
            semicolon: SemicolonStrategy::Auto,
            quote: QuoteStrategy::Auto,
            preserve_comments: CommentPreservation::None,
            source_map: SourceMapMode::None,
            source_root: None,
            include_sources_content: false,
            mapping_granularity: MappingGranularity::Token,
            newline: NewlineStyle::Lf,
            max_line_len: None,
            charset_escapes: CharsetEscapes::Minimal,
        }
    }
}

impl Generator {
    /// Create a new generator with the specified configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Generator configuration options
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rjs_compiler::generator::{Generator, GeneratorConfig};
    ///
    /// let config = GeneratorConfig::default();
    /// let generator = Generator::new(config);
    /// ```
    pub fn new(config: GeneratorConfig) -> Self {
        Self { config }
    }

    /// Generate JavaScript code from an AST
    ///
    /// # Arguments
    ///
    /// * `program` - The AST program to generate code from
    /// * `original_source` - Optional original source for source maps
    ///
    /// # Returns
    ///
    /// Returns a `GeneratorResult` containing the generated code, optional source map,
    /// and generation diagnostics.
    ///
    /// # Errors
    ///
    /// This function returns an error if:
    /// - The AST is malformed or contains unsupported nodes
    /// - Source map generation fails
    /// - I/O errors occur during generation
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use rjs_compiler::generator::{Generator, GeneratorConfig};
    /// use rjs_compiler::parser::ast_types::Program;
    ///
    /// let config = GeneratorConfig::default();
    /// let generator = Generator::new(config);
    /// let program = Program { body: vec![], source_type: ProgramSourceType::Script };
    ///
    /// match generator.generate(&program, None) {
    ///     Ok(result) => println!("Generated: {}", result.code),
    ///     Err(e) => eprintln!("Generation failed: {}", e),
    /// }
    /// ```
    pub fn generate(
        &self,
        program: &Program,
        original_source: Option<&str>,
    ) -> GeneratorResult<GenerationResult> {
        let start_time = std::time::Instant::now();
        
        // Initialize printer with configuration
        let mut printer = printer::Printer::new(&self.config);
        
        // Generate code from AST
        let code = printer.print_program(program)?;
        
        // Generate source map if requested
        let source_map = if matches!(self.config.source_map, SourceMapMode::None) {
            None
        } else {
            Some(self.generate_source_map(program, &code, original_source)?)
        };
        
        // Calculate diagnostics
        let generation_time = start_time.elapsed();
        let original_size = original_source.map(|s| s.len()).unwrap_or(0);
        let generated_size = code.len();
        let compression_ratio = if original_size > 0 {
            1.0 - (generated_size as f64 / original_size as f64)
        } else {
            0.0
        };
        
        let diagnostics = GeneratorDiagnostics {
            original_size,
            generated_size,
            compression_ratio,
            generation_time_ms: generation_time.as_secs_f64() * 1000.0,
            warning_count: printer.get_warnings().len(),
            warnings: printer.get_warnings(),
        };
        
        Ok(GenerationResult {
            code,
            source_map,
            diagnostics,
        })
    }
    
    /// Generate source map for the given program and generated code
    fn generate_source_map(
        &self,
        _program: &Program,
        _code: &str,
        _original_source: Option<&str>,
    ) -> GeneratorResult<source_maps::SourceMap> {
        // TODO: Implement source map generation
        // This is a placeholder implementation
        Ok(source_maps::SourceMap::new())
    }
}

/// CLI integration for generator configuration
impl GeneratorConfig {
    /// Create configuration from CLI arguments
    ///
    /// # Arguments
    ///
    /// * `ecma` - ECMAScript version string ("es5", "es2015", "latest")
    /// * `format` - Format string ("compact", "readable", "pretty")
    /// * `semicolon` - Semicolon strategy ("auto", "always", "remove")
    /// * `quote` - Quote strategy ("auto", "single", "double")
    /// * `preserve_comments` - Comment preservation ("none", "license", "all")
    /// * `source_map` - Source map mode ("none", "file", "inline", "indexed")
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rjs_compiler::generator::GeneratorConfig;
    ///
    /// let config = GeneratorConfig::from_cli_args(
    ///     "latest",
    ///     "compact",
    ///     "auto",
    ///     "single",
    ///     "license",
    ///     "file"
    /// );
    /// ```
    pub fn from_cli_args(
        ecma: &str,
        format: &str,
        semicolon: &str,
        quote: &str,
        preserve_comments: &str,
        source_map: &str,
    ) -> Self {
        let mut config = Self::default();
        
        config.ecma = match ecma {
            "es5" => EcmaScriptVersion::ES5,
            "es2015" | "2015" => EcmaScriptVersion::ES2015,
            "latest" | _ => EcmaScriptVersion::Latest,
        };
        
        config.format = match format {
            "compact" => OutputFormat::Compact,
            "readable" => OutputFormat::Readable,
            "pretty" => OutputFormat::Pretty,
            _ => OutputFormat::Compact,
        };
        
        config.semicolon = match semicolon {
            "auto" => SemicolonStrategy::Auto,
            "always" => SemicolonStrategy::Always,
            "remove" => SemicolonStrategy::Remove,
            _ => SemicolonStrategy::Auto,
        };
        
        config.quote = match quote {
            "auto" => QuoteStrategy::Auto,
            "single" => QuoteStrategy::Single,
            "double" => QuoteStrategy::Double,
            _ => QuoteStrategy::Auto,
        };
        
        config.preserve_comments = match preserve_comments {
            "none" => CommentPreservation::None,
            "license" => CommentPreservation::License,
            "all" => CommentPreservation::All,
            _ => CommentPreservation::None,
        };
        
        config.source_map = match source_map {
            "none" => SourceMapMode::None,
            "file" => SourceMapMode::File,
            "inline" => SourceMapMode::Inline,
            "indexed" => SourceMapMode::Indexed,
            _ => SourceMapMode::None,
        };
        
        config
    }
}
