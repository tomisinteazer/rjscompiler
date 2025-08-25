//! # Code Transformer Component
//!
//! The transformer component applies aggressive minification transformations to the analyzed AST
//! while preserving JavaScript semantics and functionality. This phase follows a strict TDD-first
//! workflow to ensure correctness and maintainability.
//!
//! ## Transformation Phases
//!
//! 1. **Identifier Renaming** - Variable and function name mangling
//! 2. **Dead Code Elimination** - Remove unused and unreachable code
//! 3. **Expression Simplification** - Constant folding and algebraic simplifications
//! 4. **Property Minification** - Safe property renaming
//! 5. **Function Minification** - Function inlining and optimization
//!
//! ## Test-Driven Development Approach
//!
//! Each transformation pass is implemented following strict TDD:
//! 1. Write failing tests for the transformation
//! 2. Implement minimal code to pass tests
//! 3. Refactor while keeping tests passing
//! 4. Add edge case tests and repeat
//!
//! ## Safety Guarantees
//!
//! All transformations preserve JavaScript semantics:
//! - Execution order maintained
//! - Side effects preserved
//! - Type coercion behavior kept
//! - This binding maintained
//! - Scope integrity preserved

use crate::analyzer::SemanticAnalysis;
use crate::parser::ast_types::Program;
use std::collections::HashMap;
use thiserror::Error;

// Re-export submodules
pub mod identifier_renaming;
pub mod dead_code_elimination;
pub mod expression_simplification;
pub mod property_minification;
pub mod function_minification;
pub mod rollback;

use crate::transformer::rollback::{RollbackManager, RollbackConfig};

#[cfg(test)]
mod tests;

/// Configuration for the transformer component
#[derive(Debug, Clone)]
pub struct TransformerConfig {
    /// Enable identifier renaming (variable mangling)
    pub enable_identifier_renaming: bool,
    /// Enable dead code elimination
    pub enable_dead_code_elimination: bool,
    /// Enable expression simplification and constant folding
    pub enable_expression_simplification: bool,
    /// Enable property minification
    pub enable_property_minification: bool,
    /// Enable function minification and inlining
    pub enable_function_minification: bool,
    /// Enable rollback for unsafe transformations
    pub enable_rollback: bool,
    /// Enable verbose output for debugging
    pub verbose: bool,
    /// Enable aggressive optimization (may be less safe)
    pub aggressive_optimization: bool,
}

impl Default for TransformerConfig {
    fn default() -> Self {
        Self {
            enable_identifier_renaming: true,
            enable_dead_code_elimination: true,
            enable_expression_simplification: true,
            enable_property_minification: true,
            enable_function_minification: true,
            enable_rollback: true,
            verbose: false,
            aggressive_optimization: false,
        }
    }
}

/// Errors that can occur during transformation
#[derive(Debug, Error)]
pub enum TransformError {
    #[error("Identifier renaming failed: {0}")]
    IdentifierRenamingError(String),
    
    #[error("Dead code elimination failed: {0}")]
    DeadCodeEliminationError(String),
    
    #[error("Expression simplification failed: {0}")]
    ExpressionSimplificationError(String),
    
    #[error("Property minification failed: {0}")]
    PropertyMinificationError(String),
    
    #[error("Function minification failed: {0}")]
    FunctionMinificationError(String),
    
    #[error("Transformation rollback required: {0}")]
    RollbackRequired(String),
    
    #[error("Invalid transformation state: {0}")]
    InvalidState(String),
}

/// Result type for transformer operations
pub type TransformResult<T> = Result<T, TransformError>;

/// Statistics about the transformation process
#[derive(Debug, Clone, Default)]
pub struct TransformationStats {
    /// Number of identifiers renamed
    pub identifiers_renamed: u32,
    /// Amount of dead code removed (in statements)
    pub dead_statements_removed: u32,
    /// Number of expressions simplified
    pub expressions_simplified: u32,
    /// Number of properties renamed
    pub properties_renamed: u32,
    /// Number of functions inlined
    pub functions_inlined: u32,
    /// Number of transformations rolled back due to safety concerns
    pub rollbacks_performed: u32,
    /// Total time spent on transformation (in milliseconds)
    pub transformation_time_ms: u64,
}

/// Result of the transformation process
#[derive(Debug, Clone)]
pub struct TransformationResult {
    /// The transformed AST
    pub transformed_ast: Program,
    /// Statistics about the transformations performed
    pub stats: TransformationStats,
    /// Mapping from original identifiers to renamed ones
    pub identifier_mapping: HashMap<String, String>,
    /// Any warnings generated during transformation
    pub warnings: Vec<String>,
}

/// Main transformer that orchestrates all transformation passes
#[derive(Debug)]
pub struct Transformer {
    /// Configuration for transformation passes
    config: TransformerConfig,
    /// Analysis result from the analyzer phase
    analysis_result: SemanticAnalysis,
    /// Rollback manager for unsafe transformations
    rollback_manager: RollbackManager,
}

impl Transformer {
    /// Creates a new transformer with the given configuration and analysis results
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration settings for transformation passes
    /// * `analysis_result` - Results from the semantic analysis phase
    ///
    /// # Returns
    ///
    /// A new `Transformer` instance ready to transform ASTs
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use rjs_compiler::transformer::{Transformer, TransformerConfig};
    /// use rjs_compiler::analyzer::SemanticAnalysis;
    /// 
    /// let config = TransformerConfig::default();
    /// let analysis_result = SemanticAnalysis::default(); // From analyzer
    /// let transformer = Transformer::new(config, analysis_result);
    /// ```
    pub fn new(config: TransformerConfig, analysis_result: SemanticAnalysis) -> Self {
        let rollback_config = RollbackConfig {
            auto_rollback: config.enable_rollback,
            max_checkpoints: 10,
            verbose: config.verbose,
        };
        let rollback_manager = RollbackManager::new(rollback_config);
        
        Self {
            config,
            analysis_result,
            rollback_manager,
        }
    }

    /// Transforms the given AST through all enabled transformation passes
    ///
    /// # Arguments
    ///
    /// * `ast` - The abstract syntax tree to transform
    ///
    /// # Returns
    ///
    /// Returns a `TransformResult<TransformationResult>` containing the transformed AST
    /// and transformation statistics, or an error if transformation fails.
    ///
    /// # Errors
    ///
    /// Returns `TransformError` if any transformation pass fails or if rollback is required.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use rjs_compiler::parser::ast_types::Program;
    /// 
    /// let ast = Program { /* ... */ };
    /// let result = transformer.transform(ast)?;
    /// println!("Transformed {} identifiers", result.stats.identifiers_renamed);
    /// ```
    pub fn transform(&mut self, mut ast: Program) -> TransformResult<TransformationResult> {
        let start_time = std::time::Instant::now();
        let mut stats = TransformationStats::default();
        let mut identifier_mapping = HashMap::new();
        let mut warnings = Vec::new();

        if self.config.verbose {
            println!("🔄 Starting transformation phase with {} passes enabled", 
                self.count_enabled_passes());
        }

        // Pass 1: Identifier Renaming (Variable Mangling)
        if self.config.enable_identifier_renaming {
            if self.config.verbose {
                println!("🔄 Pass 1: Identifier Renaming");
            }
            
            // Create checkpoint for rollback if needed
            if self.config.enable_rollback {
                self.rollback_manager.create_checkpoint(
                    &ast, 
                    "identifier_renaming", 
                    "Before identifier renaming transformation"
                );
            }
            
            let rename_result = identifier_renaming::rename_identifiers(
                &mut ast, 
                &self.analysis_result.symbol_table,
                &self.config
            )?;
            
            stats.identifiers_renamed = rename_result.renamed_count;
            identifier_mapping.extend(rename_result.mapping);
            warnings.extend(rename_result.warnings);
        }

        // Pass 2: Dead Code Elimination
        if self.config.enable_dead_code_elimination {
            if self.config.verbose {
                println!("🔄 Pass 2: Dead Code Elimination");
            }
            
            // Create checkpoint for rollback if needed
            if self.config.enable_rollback {
                self.rollback_manager.create_checkpoint(
                    &ast, 
                    "dead_code_elimination", 
                    "Before dead code elimination transformation"
                );
            }
            
            let dce_result = dead_code_elimination::eliminate_dead_code(
                &mut ast,
                &self.analysis_result.symbol_table,
                &self.config
            )?;
            
            stats.dead_statements_removed = dce_result.removed_count;
            warnings.extend(dce_result.warnings);
        }

        // Pass 3: Expression Simplification & Compression
        if self.config.enable_expression_simplification {
            if self.config.verbose {
                println!("🔄 Pass 3: Expression Simplification");
            }
            
            // Create checkpoint for rollback if needed
            if self.config.enable_rollback {
                self.rollback_manager.create_checkpoint(
                    &ast, 
                    "expression_simplification", 
                    "Before expression simplification transformation"
                );
            }
            
            let simplify_result = expression_simplification::simplify_expressions(
                &mut ast,
                &self.config
            )?;
            
            stats.expressions_simplified = simplify_result.simplified_count;
            stats.rollbacks_performed += simplify_result.rollbacks;
            warnings.extend(simplify_result.warnings);
        }

        // Pass 4: Property Minification
        if self.config.enable_property_minification {
            if self.config.verbose {
                println!("🔄 Pass 4: Property Minification");
            }
            
            let prop_result = property_minification::minify_properties(
                &mut ast,
                &self.analysis_result,
                &self.config
            )?;
            
            stats.properties_renamed = prop_result.renamed_count;
            warnings.extend(prop_result.warnings);
        }

        // Pass 5: Function Minification
        if self.config.enable_function_minification {
            if self.config.verbose {
                println!("🔄 Pass 5: Function Minification");
            }
            
            let func_result = function_minification::minify_functions(
                &mut ast,
                &self.analysis_result,
                &self.config
            )?;
            
            stats.functions_inlined = func_result.inlined_count;
            warnings.extend(func_result.warnings);
        }

        stats.transformation_time_ms = start_time.elapsed().as_millis() as u64;

        if self.config.verbose {
            println!("✅ Transformation completed in {}ms", stats.transformation_time_ms);
            self.print_transformation_summary(&stats);
        }

        Ok(TransformationResult {
            transformed_ast: ast,
            stats,
            identifier_mapping,
            warnings,
        })
    }

    /// Counts the number of enabled transformation passes
    fn count_enabled_passes(&self) -> u32 {
        let mut count = 0;
        if self.config.enable_identifier_renaming { count += 1; }
        if self.config.enable_dead_code_elimination { count += 1; }
        if self.config.enable_expression_simplification { count += 1; }
        if self.config.enable_property_minification { count += 1; }
        if self.config.enable_function_minification { count += 1; }
        count
    }

    /// Prints a summary of transformation statistics
    fn print_transformation_summary(&self, stats: &TransformationStats) {
        println!("📊 Transformation Summary:");
        println!("   🏷️  Identifiers renamed: {}", stats.identifiers_renamed);
        println!("   🗑️  Dead statements removed: {}", stats.dead_statements_removed);
        println!("   🔧 Expressions simplified: {}", stats.expressions_simplified);
        println!("   🏠 Properties renamed: {}", stats.properties_renamed);
        println!("   📎 Functions inlined: {}", stats.functions_inlined);
        
        if stats.rollbacks_performed > 0 {
            println!("   ↩️  Rollbacks performed: {}", stats.rollbacks_performed);
        }
        
        println!("   ⏱️  Total time: {}ms", stats.transformation_time_ms);
    }
}

/// Convenience function to transform an AST with default configuration
///
/// # Arguments
///
/// * `ast` - The abstract syntax tree to transform
/// * `analysis_result` - Results from the semantic analysis phase
///
/// # Returns
///
/// Returns a `TransformResult<TransformationResult>` containing the transformed AST
/// and transformation statistics.
///
/// # Examples
///
/// ```rust,no_run
/// use rjs_compiler::transformer::transform_ast;
/// use rjs_compiler::parser::ast_types::Program;
/// use rjs_compiler::analyzer::SemanticAnalysis;
/// 
/// let ast = Program { /* ... */ };
/// let analysis_result = SemanticAnalysis::default();
/// let result = transform_ast(ast, analysis_result)?;
/// ```
pub fn transform_ast(
    ast: Program, 
    analysis_result: SemanticAnalysis
) -> TransformResult<TransformationResult> {
    let config = TransformerConfig::default();
    let mut transformer = Transformer::new(config, analysis_result);
    transformer.transform(ast)
}