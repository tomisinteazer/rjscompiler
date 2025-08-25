//! # Scope Analyzer Component
//!
//! The analyzer component performs semantic analysis of the JavaScript AST, including
//! scope construction, symbol binding, capture detection, and safety classification
//! for minification. It enriches the AST with scope and semantic metadata to ensure
//! all variables, functions, and classes are correctly resolved.
//!
//! ## Key Components
//!
//! - **Scope Builder**: Constructs hierarchical scope tree for variable resolution
//! - **Symbol Table**: Tracks all identifiers and their bindings with detailed metadata
//! - **Reference Tracking**: Maps variable uses to declarations with read/write classification
//! - **Capture Detection**: Identifies closure captures for safe minification
//! - **Safety Classification**: Flags scopes and symbols that cannot be safely renamed
//!
//! ## Usage
//!
//! ```rust
//! use crate::analyzer::{analyze_ast, AnalyzerConfig};
//! use crate::parser::ast_types::Program;
//!
//! let ast = parse_javascript_code(source)?;
//! let config = AnalyzerConfig::default();
//! let analysis_result = analyze_ast(&ast, &config)?;
//! ```

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::parser::ast_types::Program;

pub mod scope_builder;
pub mod semantic_analysis;

#[cfg(test)]
mod tests;

/// Configuration for the analyzer component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzerConfig {
    /// Enable verbose analysis output
    pub verbose:                bool,
    /// Preserve export symbols (don't rename)
    pub preserve_exports:       bool,
    /// Enable aggressive optimization (may break some edge cases)
    pub aggressive_optimization: bool,
    /// Enable strict mode analysis
    pub strict_mode:            bool,
}

/// Unique identifier for scopes within the analysis
pub type ScopeId = u32;

/// Unique identifier for symbols within the analysis
pub type SymbolId = u32;

/// Errors that can occur during semantic analysis
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum AnalysisError {
    #[error("Scope analysis failed: {message}")]
    ScopeAnalysisFailed { message: String },

    #[error("Symbol resolution failed for identifier '{identifier}' at {location}")]
    SymbolResolutionFailed {
        identifier: String,
        location:   String,
    },

    #[error("Invalid scope nesting: {details}")]
    InvalidScopeNesting { details: String },

    #[error("Temporal dead zone violation: '{identifier}' used before declaration")]
    TemporalDeadZoneViolation { identifier: String },

    #[error("Unsafe scope detected: {reason}")]
    UnsafeScope { reason: String },

    #[error("Internal analysis error: {message}")]
    InternalError { message: String },
}

/// Result type for analysis operations
pub type AnalysisResult<T> = Result<T, AnalysisError>;

/// Complete analysis result containing all semantic information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticAnalysis {
    /// Symbol table containing all bindings and references
    pub symbol_table:   SymbolTable,
    /// Scope tree with hierarchical relationships
    pub scope_tree:     ScopeTree,
    /// Semantic flags for safety classification
    pub semantic_flags: SemanticFlags,
    /// Analysis metadata and statistics
    pub metadata:       AnalysisMetadata,
}

/// Symbol table tracking all identifiers and their bindings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolTable {
    /// All symbols indexed by their unique ID
    pub symbols:     HashMap<SymbolId, Symbol>,
    /// Mapping from identifier names to symbol IDs in each scope
    pub scope_bindings: HashMap<ScopeId, HashMap<String, SymbolId>>,
    /// Next available symbol ID
    pub next_symbol_id: SymbolId,
}

/// Hierarchical scope tree structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeTree {
    /// All scopes indexed by their unique ID
    pub scopes:        HashMap<ScopeId, Scope>,
    /// Root scope ID (typically global scope)
    pub root_scope_id: ScopeId,
    /// Next available scope ID
    pub next_scope_id: ScopeId,
}

/// Semantic flags for optimization safety
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticFlags {
    /// Scopes that are unsafe for renaming due to eval, with, etc.
    pub unsafe_scopes:     HashMap<ScopeId, UnsafeReason>,
    /// Symbols that cannot be safely renamed
    pub unsafe_symbols:    HashMap<SymbolId, UnsafeReason>,
    /// Global scope references that must be preserved
    pub global_references: Vec<SymbolId>,
}

/// Analysis metadata and statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisMetadata {
    /// Total number of scopes analyzed
    pub scope_count:      u32,
    /// Total number of symbols found
    pub symbol_count:     u32,
    /// Number of closure captures detected
    pub capture_count:    u32,
    /// Number of export symbols preserved
    pub export_count:     u32,
    /// Analysis time in milliseconds
    pub analysis_time_ms: u64,
}

/// Individual scope information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scope {
    /// Unique scope identifier
    pub id:          ScopeId,
    /// Type of scope (function, block, global, etc.)
    pub scope_type:  ScopeType,
    /// Parent scope ID (None for root scope)
    pub parent_id:   Option<ScopeId>,
    /// Child scope IDs
    pub children:    Vec<ScopeId>,
    /// Symbols declared directly in this scope
    pub bindings:    Vec<SymbolId>,
    /// Whether this scope is safe for aggressive optimization
    pub is_safe:     bool,
}

/// Types of scopes in JavaScript
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScopeType {
    /// Global scope (top-level)
    Global,
    /// Function scope (function declarations and expressions)
    Function,
    /// Block scope (let/const in blocks)
    Block,
    /// Module scope (ES6 modules)
    Module,
    /// Class scope (class declarations)
    Class,
    /// Catch scope (exception parameter)
    Catch,
    /// With scope (with statements - always unsafe)
    With,
}

/// Individual symbol information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    /// Unique symbol identifier
    pub id:          SymbolId,
    /// Original identifier name
    pub name:        String,
    /// Type of symbol
    pub symbol_type: SymbolType,
    /// Scope where this symbol is declared
    pub scope_id:    ScopeId,
    /// All references to this symbol
    pub references:  Vec<SymbolReference>,
    /// Whether this symbol is captured by closures
    pub is_captured: bool,
    /// Whether this symbol is exported (ES6 modules)
    pub is_exported: bool,
    /// Whether this symbol can be safely renamed
    pub is_renamable: bool,
}

/// Types of symbols in JavaScript
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SymbolType {
    /// Variable declaration (let, const, var)
    Variable { kind: VariableKind },
    /// Function declaration or expression
    Function,
    /// Class declaration or expression
    Class,
    /// Function or method parameter
    Parameter,
    /// Import binding (ES6 modules)
    Import,
    /// Export binding (ES6 modules)
    Export,
    /// Property name (when safe to rename)
    Property,
}

/// Variable declaration kinds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VariableKind {
    /// var declaration (function-scoped)
    Var,
    /// let declaration (block-scoped)
    Let,
    /// const declaration (block-scoped, immutable)
    Const,
}

/// Reference to a symbol with usage context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolReference {
    /// Location in the source code
    pub location:       SourceLocation,
    /// Type of reference (read, write, call, etc.)
    pub reference_type: ReferenceType,
    /// Scope where the reference occurs
    pub scope_id:       ScopeId,
}

/// Types of symbol references
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReferenceType {
    /// Reading the symbol value
    Read,
    /// Writing to the symbol
    Write,
    /// Calling the symbol as a function
    Call,
    /// Accessing as property
    PropertyAccess,
}

/// Source code location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLocation {
    /// Line number (1-based)
    pub line:   u32,
    /// Column number (0-based)
    pub column: u32,
    /// Character offset in source
    pub offset: u32,
}

/// Reasons why a scope or symbol is unsafe for optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UnsafeReason {
    /// Contains eval() call
    EvalUsage,
    /// Contains with statement
    WithStatement,
    /// Dynamic this binding
    DynamicThis,
    /// Indirect variable access (e.g., window['variable'])
    IndirectAccess,
    /// External module dependency
    ExternalDependency,
    /// Unknown safety (conservative approach)
    Unknown,
}

impl Default for AnalyzerConfig {
    fn default() -> Self {
        Self {
            verbose:                false,
            preserve_exports:       true,
            aggressive_optimization: false,
            strict_mode:            true,
        }
    }
}

impl SymbolTable {
    /// Creates a new empty symbol table
    pub fn new() -> Self {
        Self {
            symbols:        HashMap::new(),
            scope_bindings: HashMap::new(),
            next_symbol_id: 0,
        }
    }

    /// Generates the next unique symbol ID
    pub fn next_id(&mut self) -> SymbolId {
        let id = self.next_symbol_id;
        self.next_symbol_id += 1;
        id
    }
}

impl ScopeTree {
    /// Creates a new scope tree with a root scope
    pub fn new(root_scope_type: ScopeType) -> Self {
        let mut scopes = HashMap::new();
        let root_scope = Scope {
            id:          0,
            scope_type:  root_scope_type,
            parent_id:   None,
            children:    Vec::new(),
            bindings:    Vec::new(),
            is_safe:     true,
        };
        scopes.insert(0, root_scope);

        Self {
            scopes,
            root_scope_id: 0,
            next_scope_id: 1,
        }
    }

    /// Generates the next unique scope ID
    pub fn next_id(&mut self) -> ScopeId {
        let id = self.next_scope_id;
        self.next_scope_id += 1;
        id
    }

    /// Gets a scope by ID
    pub fn get_scope(&self, scope_id: ScopeId) -> Option<&Scope> {
        self.scopes.get(&scope_id)
    }

    /// Gets a mutable scope by ID
    pub fn get_scope_mut(&mut self, scope_id: ScopeId) -> Option<&mut Scope> {
        self.scopes.get_mut(&scope_id)
    }
}

/// Main analysis function that performs semantic analysis on the AST.
///
/// # Arguments
///
/// * `ast` - The JavaScript AST to analyze
/// * `config` - Configuration options for analysis
///
/// # Returns
///
/// Returns a `SemanticAnalysis` result containing symbol table, scope tree,
/// and semantic flags, or an `AnalysisError` if analysis fails.
///
/// # Examples
///
/// ```rust
/// use crate::analyzer::{analyze_ast, AnalyzerConfig};
///
/// let config = AnalyzerConfig::default();
/// let analysis = analyze_ast(&ast, &config)?;
/// println!("Found {} symbols in {} scopes", 
///          analysis.metadata.symbol_count,
///          analysis.metadata.scope_count);
/// ```
pub fn analyze_ast(ast: &Program, config: &AnalyzerConfig) -> AnalysisResult<SemanticAnalysis> {
    let start_time = std::time::Instant::now();

    if config.verbose {
        println!("Starting semantic analysis...");
    }

    // Initialize analysis components
    let mut symbol_table = SymbolTable::new();
    let mut scope_tree = ScopeTree::new(ScopeType::Global);
    let mut semantic_flags = SemanticFlags {
        unsafe_scopes:     HashMap::new(),
        unsafe_symbols:    HashMap::new(),
        global_references: Vec::new(),
    };

    // Perform scope analysis
    scope_builder::analyze_scopes(
        ast,
        &mut scope_tree,
        &mut symbol_table,
        &mut semantic_flags,
        config,
    )?;

    // Perform semantic analysis
    semantic_analysis::analyze_semantics(
        ast,
        &mut scope_tree,
        &mut symbol_table,
        &mut semantic_flags,
        config,
    )?;

    let analysis_time = start_time.elapsed().as_millis() as u64;

    let metadata = AnalysisMetadata {
        scope_count:      scope_tree.next_scope_id,
        symbol_count:     symbol_table.next_symbol_id,
        capture_count:    symbol_table
            .symbols
            .values()
            .filter(|s| s.is_captured)
            .count() as u32,
        export_count:     symbol_table
            .symbols
            .values()
            .filter(|s| s.is_exported)
            .count() as u32,
        analysis_time_ms: analysis_time,
    };

    if config.verbose {
        println!(
            "Analysis completed in {}ms: {} scopes, {} symbols",
            analysis_time, metadata.scope_count, metadata.symbol_count
        );
    }

    Ok(SemanticAnalysis {
        symbol_table,
        scope_tree,
        semantic_flags,
        metadata,
    })
}