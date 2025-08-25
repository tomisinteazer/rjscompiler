//! # Rollback Module
//!
//! Handles rollback of unsafe transformations that could change runtime behavior.
//! This module implements safety checks and rollback mechanisms to ensure semantic
//! preservation during aggressive optimization.

use crate::analyzer::SemanticAnalysis;
use crate::parser::ast_types::{Expression, Program, Statement};
use crate::transformer::{TransformError, TransformResult, TransformerConfig};
use std::collections::HashMap;

/// Stores the original state of a transformation for potential rollback
#[derive(Debug, Clone)]
pub struct TransformationCheckpoint {
    /// Original AST state before transformation
    pub original_ast: Program,
    /// Transformation pass identifier
    pub pass_name: String,
    /// Reason for creating the checkpoint
    pub reason: String,
}

/// Manages rollback operations for unsafe transformations
#[derive(Debug)]
pub struct RollbackManager {
    /// Stack of transformation checkpoints
    checkpoints: Vec<TransformationCheckpoint>,
    /// Configuration for rollback behavior
    config: RollbackConfig,
}

/// Configuration for rollback behavior
#[derive(Debug, Clone)]
pub struct RollbackConfig {
    /// Enable automatic rollback on semantic violations
    pub auto_rollback: bool,
    /// Maximum number of checkpoints to maintain
    pub max_checkpoints: usize,
    /// Enable verbose rollback logging
    pub verbose: bool,
}

impl Default for RollbackConfig {
    fn default() -> Self {
        Self {
            auto_rollback: true,
            max_checkpoints: 10,
            verbose: false,
        }
    }
}

impl RollbackManager {
    /// Creates a new rollback manager
    pub fn new(config: RollbackConfig) -> Self {
        Self {
            checkpoints: Vec::new(),
            config,
        }
    }

    /// Creates a checkpoint before a potentially unsafe transformation
    ///
    /// # Arguments
    ///
    /// * `ast` - Current AST state to checkpoint
    /// * `pass_name` - Name of the transformation pass
    /// * `reason` - Reason for creating the checkpoint
    pub fn create_checkpoint(
        &mut self,
        ast: &Program,
        pass_name: &str,
        reason: &str,
    ) {
        if self.config.verbose {
            println!("📍 Creating checkpoint for {}: {}", pass_name, reason);
        }

        let checkpoint = TransformationCheckpoint {
            original_ast: ast.clone(),
            pass_name: pass_name.to_string(),
            reason: reason.to_string(),
        };

        self.checkpoints.push(checkpoint);

        // Maintain maximum checkpoint limit
        if self.checkpoints.len() > self.config.max_checkpoints {
            self.checkpoints.remove(0);
        }
    }

    /// Rolls back to the most recent checkpoint
    ///
    /// # Returns
    ///
    /// Returns the original AST state or an error if no checkpoints exist
    pub fn rollback_to_last_checkpoint(&mut self) -> TransformResult<Program> {
        if let Some(checkpoint) = self.checkpoints.pop() {
            if self.config.verbose {
                println!("↩️ Rolling back transformation: {} ({})", 
                    checkpoint.pass_name, checkpoint.reason);
            }
            Ok(checkpoint.original_ast)
        } else {
            Err(TransformError::RollbackRequired(
                "No checkpoints available for rollback".to_string()
            ))
        }
    }

    /// Rolls back to a specific checkpoint by pass name
    ///
    /// # Arguments
    ///
    /// * `pass_name` - Name of the transformation pass to roll back to
    pub fn rollback_to_pass(&mut self, pass_name: &str) -> TransformResult<Program> {
        // Find the checkpoint for the specified pass
        if let Some(pos) = self.checkpoints.iter().rposition(|cp| cp.pass_name == pass_name) {
            let checkpoint = self.checkpoints.remove(pos);
            // Remove all checkpoints after this one
            self.checkpoints.truncate(pos);
            
            if self.config.verbose {
                println!("↩️ Rolling back to pass: {} ({})", 
                    checkpoint.pass_name, checkpoint.reason);
            }
            Ok(checkpoint.original_ast)
        } else {
            Err(TransformError::RollbackRequired(
                format!("No checkpoint found for pass: {}", pass_name)
            ))
        }
    }

    /// Validates a transformation by checking for semantic violations
    ///
    /// # Arguments
    ///
    /// * `original_ast` - AST before transformation
    /// * `transformed_ast` - AST after transformation
    /// * `analysis` - Semantic analysis results
    ///
    /// # Returns
    ///
    /// Returns true if the transformation is safe, false if rollback is needed
    pub fn validate_transformation(
        &self,
        original_ast: &Program,
        transformed_ast: &Program,
        analysis: &SemanticAnalysis,
    ) -> bool {
        // Check for unsafe transformations
        if self.has_unsafe_constant_folding(original_ast, transformed_ast) {
            return false;
        }

        if self.has_unsafe_function_inlining(original_ast, transformed_ast, analysis) {
            return false;
        }

        if self.has_unsafe_variable_elimination(original_ast, transformed_ast, analysis) {
            return false;
        }

        true
    }

    /// Checks for unsafe constant folding that could change runtime behavior
    fn has_unsafe_constant_folding(
        &self,
        _original_ast: &Program,
        _transformed_ast: &Program,
    ) -> bool {
        // TODO: Implement detection of unsafe constant folding
        // Examples:
        // - Math.random() should not be folded
        // - Date.now() should not be folded
        // - Division by zero should not be folded
        // - NaN operations should be preserved
        false
    }

    /// Checks for unsafe function inlining
    fn has_unsafe_function_inlining(
        &self,
        _original_ast: &Program,
        _transformed_ast: &Program,
        _analysis: &SemanticAnalysis,
    ) -> bool {
        // TODO: Implement detection of unsafe function inlining
        // Examples:
        // - Functions with side effects should not be inlined
        // - Recursive functions should not be inlined
        // - Functions that access 'this' should be carefully handled
        false
    }

    /// Checks for unsafe variable elimination
    fn has_unsafe_variable_elimination(
        &self,
        _original_ast: &Program,
        _transformed_ast: &Program,
        _analysis: &SemanticAnalysis,
    ) -> bool {
        // TODO: Implement detection of unsafe variable elimination
        // Examples:
        // - Variables accessed by eval should not be eliminated
        // - Variables in closures should be preserved
        // - Exported variables should not be eliminated
        false
    }

    /// Clears all checkpoints
    pub fn clear_checkpoints(&mut self) {
        if self.config.verbose && !self.checkpoints.is_empty() {
            println!("🗑️ Clearing {} checkpoints", self.checkpoints.len());
        }
        self.checkpoints.clear();
    }

    /// Returns the number of active checkpoints
    pub fn checkpoint_count(&self) -> usize {
        self.checkpoints.len()
    }
}

/// Utility function to perform a safe transformation with automatic rollback
///
/// # Arguments
///
/// * `ast` - AST to transform
/// * `pass_name` - Name of the transformation pass
/// * `transform_fn` - Function that performs the transformation
/// * `analysis` - Semantic analysis results
/// * `rollback_manager` - Rollback manager instance
///
/// # Returns
///
/// Returns the transformed AST or the original AST if rollback was required
pub fn safe_transform<F>(
    mut ast: Program,
    pass_name: &str,
    transform_fn: F,
    analysis: &SemanticAnalysis,
    rollback_manager: &mut RollbackManager,
) -> TransformResult<Program>
where
    F: FnOnce(&mut Program) -> TransformResult<()>,
{
    // Create checkpoint before transformation
    rollback_manager.create_checkpoint(&ast, pass_name, "Safety checkpoint");
    
    // Store original for comparison
    let original_ast = ast.clone();
    
    // Perform transformation
    match transform_fn(&mut ast) {
        Ok(()) => {
            // Validate the transformation
            if rollback_manager.validate_transformation(&original_ast, &ast, analysis) {
                // Transformation is safe, clear the checkpoint
                Ok(ast)
            } else {
                // Transformation is unsafe, rollback
                if rollback_manager.config.verbose {
                    println!("⚠️ Unsafe transformation detected in {}, rolling back", pass_name);
                }
                rollback_manager.rollback_to_last_checkpoint()
            }
        }
        Err(e) => {
            // Transformation failed, rollback
            if rollback_manager.config.verbose {
                println!("❌ Transformation failed in {}: {}, rolling back", pass_name, e);
            }
            rollback_manager.rollback_to_last_checkpoint()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzer::{SymbolTable, SemanticFlags, AnalysisMetadata, ScopeTree, ScopeType};
    use crate::parser::ast_types::{Program, ProgramSourceType};

    fn create_test_ast() -> Program {
        Program {
            body: vec![],
            source_type: ProgramSourceType::Script,
        }
    }

    fn create_test_analysis() -> SemanticAnalysis {
        SemanticAnalysis {
            symbol_table: SymbolTable::new(),
            scope_tree: ScopeTree::new(ScopeType::Global),
            semantic_flags: SemanticFlags {
                unsafe_scopes: HashMap::new(),
                unsafe_symbols: HashMap::new(),
                global_references: Vec::new(),
            },
            metadata: AnalysisMetadata {
                scope_count: 1,
                symbol_count: 0,
                capture_count: 0,
                export_count: 0,
                analysis_time_ms: 0,
            },
        }
    }

    #[test]
    fn test_rollback_manager_creation() {
        let config = RollbackConfig::default();
        let manager = RollbackManager::new(config);
        assert_eq!(manager.checkpoint_count(), 0);
    }

    #[test]
    fn test_create_checkpoint() {
        let mut manager = RollbackManager::new(RollbackConfig::default());
        let ast = create_test_ast();
        
        manager.create_checkpoint(&ast, "test_pass", "test reason");
        assert_eq!(manager.checkpoint_count(), 1);
    }

    #[test]
    fn test_rollback_to_last_checkpoint() {
        let mut manager = RollbackManager::new(RollbackConfig::default());
        let ast = create_test_ast();
        
        manager.create_checkpoint(&ast, "test_pass", "test reason");
        let rolled_back = manager.rollback_to_last_checkpoint().unwrap();
        
        assert_eq!(rolled_back.body.len(), ast.body.len());
        assert_eq!(manager.checkpoint_count(), 0);
    }

    #[test]
    fn test_rollback_without_checkpoint() {
        let mut manager = RollbackManager::new(RollbackConfig::default());
        let result = manager.rollback_to_last_checkpoint();
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TransformError::RollbackRequired(_)));
    }

    #[test]
    fn test_rollback_to_specific_pass() {
        let mut manager = RollbackManager::new(RollbackConfig::default());
        let ast1 = create_test_ast();
        let ast2 = create_test_ast();
        
        manager.create_checkpoint(&ast1, "pass1", "reason1");
        manager.create_checkpoint(&ast2, "pass2", "reason2");
        
        let rolled_back = manager.rollback_to_pass("pass1").unwrap();
        assert_eq!(manager.checkpoint_count(), 0);
    }

    #[test]
    fn test_safe_transform_success() {
        let ast = create_test_ast();
        let analysis = create_test_analysis();
        let mut manager = RollbackManager::new(RollbackConfig::default());
        
        let result = safe_transform(
            ast,
            "test_pass",
            |_ast| Ok(()),
            &analysis,
            &mut manager,
        );
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_safe_transform_failure() {
        let ast = create_test_ast();
        let analysis = create_test_analysis();
        let mut manager = RollbackManager::new(RollbackConfig::default());
        
        let result = safe_transform(
            ast,
            "test_pass",
            |_ast| Err(TransformError::InvalidState("test error".to_string())),
            &analysis,
            &mut manager,
        );
        
        assert!(result.is_ok()); // Should rollback to original state
    }

    #[test]
    fn test_checkpoint_limit() {
        let config = RollbackConfig {
            max_checkpoints: 2,
            ..RollbackConfig::default()
        };
        let mut manager = RollbackManager::new(config);
        let ast = create_test_ast();
        
        manager.create_checkpoint(&ast, "pass1", "reason1");
        manager.create_checkpoint(&ast, "pass2", "reason2");
        manager.create_checkpoint(&ast, "pass3", "reason3"); // Should remove pass1
        
        assert_eq!(manager.checkpoint_count(), 2);
        
        // Should not be able to rollback to pass1
        let result = manager.rollback_to_pass("pass1");
        assert!(result.is_err());
    }

    #[test]
    fn test_validation_passes() {
        let manager = RollbackManager::new(RollbackConfig::default());
        let ast = create_test_ast();
        let analysis = create_test_analysis();
        
        let is_valid = manager.validate_transformation(&ast, &ast, &analysis);
        assert!(is_valid);
    }
}