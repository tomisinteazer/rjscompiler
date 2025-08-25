# Rust Documentation Standards and Guidelines

## API Documentation Requirements

### Comprehensive Function Documentation
Every public function should include complete documentation:

```rust
/// Minifies JavaScript source code while preserving functionality.
///
/// This function applies aggressive optimizations including variable renaming,
/// dead code elimination, and expression simplification to reduce the size
/// of JavaScript code by 70-90% while maintaining exact semantic behavior.
///
/// # Arguments
///
/// * `source` - The JavaScript source code to minify
/// * `config` - Configuration options controlling minification behavior
///
/// # Returns
///
/// Returns a `Result` containing the minified JavaScript code on success,
/// or a `MinifierError` if minification fails.
///
/// # Errors
///
/// This function will return an error if:
/// - The input JavaScript has syntax errors (`MinifierError::Parse`)
/// - Scope analysis fails due to complex constructs (`MinifierError::Analysis`)
/// - Transformation fails due to unsupported patterns (`MinifierError::Transform`)
/// - Code generation encounters internal errors (`MinifierError::Generation`)
///
/// # Examples
///
/// Basic minification:
/// ```rust
/// use rjs_compiler::{minify_javascript, MinifierConfig};
///
/// let source = r#"
///     function calculateSum(firstNumber, secondNumber) {
///         const result = firstNumber + secondNumber;
///         return result;
///     }
/// "#;
///
/// let config = MinifierConfig::default();
/// let minified = minify_javascript(source, &config)?;
/// 
/// // Result: "function a(b,c){const d=b+c;return d}"
/// assert!(minified.len() < source.len() * 0.3);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// Advanced configuration:
/// ```rust
/// use rjs_compiler::{minify_javascript, MinifierConfig, OptimizationLevel};
///
/// let config = MinifierConfig::builder()
///     .optimization_level(OptimizationLevel::Aggressive)
///     .preserve_license_comments(true)
///     .source_maps(true)
///     .build()?;
///
/// let minified = minify_javascript(source, &config)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Performance
///
/// Typical performance characteristics:
/// - Small files (< 10KB): < 1ms
/// - Medium files (10KB - 100KB): 1-10ms  
/// - Large files (100KB - 1MB): 10-100ms
///
/// Memory usage is approximately 2-3x the input size during processing.
///
/// # Safety
///
/// This function is safe and does not use any unsafe code. All transformations
/// preserve JavaScript semantics and maintain the original program behavior.
pub fn minify_javascript(
    source: &str,
    config: &MinifierConfig,
) -> Result<String, MinifierError> {
    // Implementation...
}
```

### Module-Level Documentation
Provide comprehensive module overviews:

```rust
//! # JavaScript Parser Module
//!
//! This module provides high-performance parsing of JavaScript source code into
//! an Abstract Syntax Tree (AST) suitable for analysis and transformation.
//!
//! ## Architecture
//!
//! The parser follows a two-stage approach:
//!
//! 1. **Lexical Analysis** ([`Lexer`]): Converts source text into tokens
//! 2. **Syntax Analysis** ([`Parser`]): Builds AST from token stream
//!
//! ## Supported JavaScript Features
//!
//! - **ES2015+**: Arrow functions, destructuring, template literals
//! - **ES2017+**: Async/await, object spread/rest
//! - **ES2020+**: Optional chaining, nullish coalescing
//! - **ES2022+**: Private class fields, top-level await
//!
//! ## Error Handling
//!
//! The parser provides detailed error reporting with:
//! - Precise location information (line, column)
//! - Descriptive error messages
//! - Error recovery for batch processing
//!
//! ## Performance
//!
//! The parser is optimized for speed and memory efficiency:
//! - Zero-copy tokenization where possible
//! - Minimal allocations during parsing
//! - Incremental parsing support for large files
//!
//! ## Examples
//!
//! Basic parsing:
//! ```rust
//! use rjs_compiler::parser::{Parser, ParseResult};
//!
//! let source = "function greet(name) { return `Hello, ${name}!`; }";
//! let mut parser = Parser::new(source);
//! let ast = parser.parse()?;
//!
//! assert_eq!(ast.kind(), AstNodeKind::Program);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! Error recovery:
//! ```rust
//! use rjs_compiler::parser::{Parser, ParseOptions};
//!
//! let source_with_errors = r#"
//!     function valid() {}
//!     function invalid( {  // Syntax error
//!     function alsoValid() {}
//! "#;
//!
//! let options = ParseOptions::builder()
//!     .error_recovery(true)
//!     .build();
//!
//! let result = Parser::with_options(source_with_errors, options).parse_all();
//! assert!(result.ast.statements.len() == 2); // Recovered 2 valid functions
//! assert!(result.errors.len() == 1);         // Captured 1 error
//! ```

use std::collections::HashMap;
use crate::ast::{AstNode, AstNodeKind};
use crate::error::{ParseError, ParseResult};
```

### Type Documentation
Document complex types thoroughly:

```rust
/// Configuration for JavaScript minification process.
///
/// This structure controls all aspects of the minification process, from
/// parsing options to output generation. It follows a builder pattern
/// for convenient construction and supports loading from configuration files.
///
/// # Configuration Layers
///
/// Configuration is applied in the following priority order:
/// 1. Explicit builder methods (highest priority)
/// 2. Configuration file settings
/// 3. Environment variables  
/// 4. Default values (lowest priority)
///
/// # Examples
///
/// Default configuration:
/// ```rust
/// use rjs_compiler::MinifierConfig;
///
/// let config = MinifierConfig::default();
/// assert_eq!(config.optimization_level(), OptimizationLevel::Safe);
/// ```
///
/// Custom configuration:
/// ```rust
/// use rjs_compiler::{MinifierConfig, OptimizationLevel, JavaScriptVersion};
///
/// let config = MinifierConfig::builder()
///     .optimization_level(OptimizationLevel::Aggressive)
///     .target_version(JavaScriptVersion::Es2020)
///     .preserve_license_comments(true)
///     .build()?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// Loading from file:
/// ```rust
/// use rjs_compiler::MinifierConfig;
///
/// let config = MinifierConfig::from_file("minifier.config.json")?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct MinifierConfig {
    /// Optimization aggressiveness level
    optimization_level: OptimizationLevel,
    
    /// Target JavaScript version for compatibility
    target_version: JavaScriptVersion,
    
    /// Whether to preserve license header comments
    preserve_license_comments: bool,
    
    /// Whether to generate source maps
    source_maps: bool,
    
    /// Maximum number of parallel processing threads
    max_threads: Option<usize>,
}

/// Represents different levels of optimization aggressiveness.
///
/// Each level represents a different trade-off between minification
/// effectiveness and transformation safety.
///
/// # Levels
///
/// - [`Safe`](Self::Safe): Conservative optimizations with maximum compatibility
/// - [`Aggressive`](Self::Aggressive): Maximum size reduction with some risk
/// - [`Custom`](Self::Custom): User-defined optimization rules
///
/// # Examples
///
/// ```rust
/// use rjs_compiler::OptimizationLevel;
///
/// // Safe for production use
/// let safe = OptimizationLevel::Safe;
///
/// // Maximum compression
/// let aggressive = OptimizationLevel::Aggressive;
///
/// // Custom rules from configuration
/// let custom = OptimizationLevel::Custom {
///     rename_variables: true,
///     remove_dead_code: false,
///     inline_functions: true,
/// };
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum OptimizationLevel {
    /// Conservative optimizations that preserve all edge case behaviors.
    ///
    /// This level applies only transformations that are guaranteed to be safe
    /// across all JavaScript environments and edge cases.
    Safe,
    
    /// Aggressive optimizations for maximum size reduction.
    ///
    /// This level may apply transformations that could theoretically change
    /// behavior in extreme edge cases, but are safe for typical JavaScript code.
    Aggressive,
    
    /// Custom optimization rules defined by the user.
    ///
    /// This level allows fine-grained control over which optimizations are applied.
    Custom {
        /// Whether to rename variables to shorter names
        rename_variables: bool,
        /// Whether to remove unreachable code
        remove_dead_code: bool,
        /// Whether to inline small functions
        inline_functions: bool,
    },
}
```

## Documentation Best Practices

### Code Examples in Documentation
Provide comprehensive, runnable examples:

```rust
/// Analyzes JavaScript code to build a symbol table for safe variable renaming.
///
/// This function performs scope analysis to identify all variable declarations,
/// their usage sites, and their scope relationships. This information is
/// essential for safe variable renaming during minification.
///
/// # Arguments
///
/// * `ast` - The parsed Abstract Syntax Tree to analyze
/// * `options` - Analysis options controlling behavior
///
/// # Returns
///
/// Returns a `SymbolTable` containing all identified symbols and their relationships.
///
/// # Examples
///
/// Basic scope analysis:
/// ```rust
/// use rjs_compiler::{parse_javascript, analyze_symbols, AnalysisOptions};
///
/// let source = r#"
///     function outer(param) {
///         var local = param * 2;
///         function inner() {
///             return local + 1;
///         }
///         return inner();
///     }
/// "#;
///
/// let ast = parse_javascript(source)?;
/// let options = AnalysisOptions::default();
/// let symbol_table = analyze_symbols(&ast, &options)?;
///
/// // Verify scope structure
/// assert_eq!(symbol_table.scope_count(), 3); // global, outer, inner
/// assert_eq!(symbol_table.symbol_count(), 4); // outer, param, local, inner
///
/// // Check symbol relationships
/// let param_symbol = symbol_table.find_symbol("param").unwrap();
/// let local_symbol = symbol_table.find_symbol("local").unwrap();
/// assert!(local_symbol.references_symbol(param_symbol.id()));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// Complex scoping example:
/// ```rust
/// use rjs_compiler::{parse_javascript, analyze_symbols, AnalysisOptions};
///
/// let source = r#"
///     let x = 1;
///     {
///         let x = 2; // Different variable due to block scoping
///         console.log(x); // References block-scoped x
///     }
///     console.log(x); // References outer x
/// "#;
///
/// let ast = parse_javascript(source)?;
/// let symbol_table = analyze_symbols(&ast, &AnalysisOptions::default())?;
///
/// // Should identify two different 'x' variables
/// let x_symbols: Vec<_> = symbol_table.find_symbols_by_name("x").collect();
/// assert_eq!(x_symbols.len(), 2);
/// assert_ne!(x_symbols[0].scope_id(), x_symbols[1].scope_id());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn analyze_symbols(
    ast: &AstNode,
    options: &AnalysisOptions,
) -> Result<SymbolTable, AnalysisError> {
    // Implementation...
}
```

### Error Documentation
Document all possible error conditions:

```rust
/// Errors that can occur during JavaScript minification.
///
/// This enum provides detailed error information for different failure modes
/// during the minification process. Each variant includes specific context
/// to help diagnose and resolve issues.
#[derive(Debug, thiserror::Error)]
pub enum MinifierError {
    /// Parsing failed due to invalid JavaScript syntax.
    ///
    /// This error occurs when the input JavaScript contains syntax errors
    /// that prevent successful parsing into an AST.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rjs_compiler::{minify_javascript, MinifierConfig, MinifierError};
    ///
    /// let invalid_js = "function test( { return 42; }"; // Missing closing paren
    /// let result = minify_javascript(invalid_js, &MinifierConfig::default());
    ///
    /// match result {
    ///     Err(MinifierError::Parse(parse_err)) => {
    ///         println!("Syntax error at line {}, column {}: {}", 
    ///                  parse_err.line(), parse_err.column(), parse_err.message());
    ///     }
    ///     _ => unreachable!(),
    /// }
    /// ```
    #[error("Parse error: {0}")]
    Parse(#[from] ParseError),
    
    /// Scope analysis failed due to complex or unsupported constructs.
    ///
    /// This error occurs when the scope analyzer encounters JavaScript
    /// constructs that are too complex to analyze safely for variable renaming.
    ///
    /// # Common Causes
    ///
    /// - Use of `eval()` function
    /// - `with` statements
    /// - Dynamic property access in complex patterns
    /// - Circular references in scope chains
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rjs_compiler::{minify_javascript, MinifierConfig, MinifierError};
    ///
    /// let complex_js = r#"
    ///     function dangerous() {
    ///         eval("var dynamicVar = 42;");
    ///         return dynamicVar; // Cannot safely analyze this
    ///     }
    /// "#;
    ///
    /// let result = minify_javascript(complex_js, &MinifierConfig::default());
    /// assert!(matches!(result, Err(MinifierError::Analysis(_))));
    /// ```
    #[error("Scope analysis error: {0}")]
    Analysis(#[from] AnalysisError),
    
    /// Code transformation failed during minification.
    ///
    /// This error occurs when a transformation pass encounters an unexpected
    /// condition or when transformations conflict with each other.
    #[error("Transformation error: {0}")]
    Transform(#[from] TransformError),
    
    /// Code generation failed to produce valid output.
    ///
    /// This error occurs during the final code generation phase when the
    /// transformed AST cannot be converted back to valid JavaScript.
    #[error("Code generation error: {0}")]
    Generation(#[from] GenerationError),
    
    /// I/O error occurred while reading input or writing output.
    ///
    /// This error wraps standard I/O errors that can occur when processing
    /// files or streams.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
```

## Internal Documentation

### Algorithm Documentation
Document complex algorithms and their trade-offs:

```rust
/// Generates short variable names using a predictable algorithm.
///
/// This function implements a base-62 naming scheme that generates the shortest
/// possible names for renamed variables. The algorithm prioritizes frequently
/// used variables by assigning them shorter names.
///
/// # Algorithm
///
/// The naming scheme works as follows:
/// 1. Single lowercase letters: a, b, c, ..., z (26 names)
/// 2. Single uppercase letters: A, B, C, ..., Z (26 names)  
/// 3. Numbers: 0, 1, 2, ..., 9 (10 names)
/// 4. Two-character combinations: aa, ab, ..., zz, aA, ..., 99
/// 5. Three-character combinations: aaa, aab, ...
///
/// # Performance
///
/// - Time complexity: O(log₆₂(index))
/// - Space complexity: O(log₆₂(index)) for the returned string
/// - Cache-friendly: Sequential indices produce predictable names
///
/// # Arguments
///
/// * `index` - Zero-based index determining which name to generate
///
/// # Returns
///
/// Returns a string containing the generated variable name.
///
/// # Examples
///
/// ```rust
/// use rjs_compiler::generate_short_name;
///
/// // First 26 names are single lowercase letters
/// assert_eq!(generate_short_name(0), "a");
/// assert_eq!(generate_short_name(25), "z");
///
/// // Next 26 names are single uppercase letters  
/// assert_eq!(generate_short_name(26), "A");
/// assert_eq!(generate_short_name(51), "Z");
///
/// // Then single digits
/// assert_eq!(generate_short_name(52), "0");
/// assert_eq!(generate_short_name(61), "9");
///
/// // Then two-character combinations
/// assert_eq!(generate_short_name(62), "aa");
/// assert_eq!(generate_short_name(63), "ab");
/// ```
///
/// # Implementation Notes
///
/// The function uses base-62 arithmetic to convert the index into a name.
/// This ensures that names are generated deterministically and that the
/// shortest possible names are always used first.
fn generate_short_name(index: usize) -> String {
    const CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    
    if index < CHARS.len() {
        // Single character names
        return (CHARS[index] as char).to_string();
    }
    
    // Multi-character names using base-62 arithmetic
    let mut result = String::new();
    let mut remaining = index;
    
    while remaining > 0 {
        result.push(CHARS[remaining % CHARS.len()] as char);
        remaining /= CHARS.len();
    }
    
    result.chars().rev().collect()
}
```

### Performance Documentation
Document performance characteristics and trade-offs:

```rust
/// High-performance symbol table optimized for minification workloads.
///
/// This implementation uses multiple data structures to optimize different
/// access patterns common in JavaScript minification:
///
/// - **HashMap**: O(1) symbol lookup by name
/// - **Vec**: Cache-friendly iteration over all symbols
/// - **BTreeMap**: Sorted access for deterministic output
///
/// # Memory Layout
///
/// The symbol table uses a structure-of-arrays layout to improve cache
/// locality during bulk operations:
///
/// ```text
/// ┌─────────────┬─────────────┬─────────────┐
/// │   Names     │    Types    │   Scopes    │
/// ├─────────────┼─────────────┼─────────────┤
/// │ "variable1" │  Variable   │  Scope(0)   │
/// │ "function1" │  Function   │  Scope(0)   │
/// │ "param1"    │  Parameter  │  Scope(1)   │
/// └─────────────┴─────────────┴─────────────┘
/// ```
///
/// # Performance Characteristics
///
/// | Operation              | Time Complexity | Space Complexity |
/// |------------------------|-----------------|------------------|
/// | Symbol lookup          | O(1) average    | O(1)            |
/// | Symbol insertion       | O(1) average    | O(1)            |
/// | Iteration over symbols | O(n)            | O(1)            |
/// | Scope-filtered access  | O(n)            | O(1)            |
/// | Memory usage          | O(n)            | O(n)            |
///
/// # Cache Performance
///
/// The implementation is optimized for modern CPU cache hierarchies:
/// - Symbol names are interned to reduce memory usage
/// - Bulk operations iterate over contiguous memory
/// - Hot paths avoid pointer chasing
///
/// # Thread Safety
///
/// This implementation is not thread-safe. For concurrent access, wrap
/// in a `Mutex` or use the thread-safe `ConcurrentSymbolTable` variant.
pub struct SymbolTable {
    // Primary storage using structure-of-arrays layout
    names: Vec<InternedString>,
    types: Vec<SymbolType>, 
    scopes: Vec<ScopeId>,
    is_used: Vec<bool>,
    
    // Secondary indices for fast lookup
    name_to_index: AHashMap<InternedString, usize>,
    scope_to_symbols: BTreeMap<ScopeId, Vec<usize>>,
    
    // String interning for memory efficiency
    string_interner: StringInterner,
}
```