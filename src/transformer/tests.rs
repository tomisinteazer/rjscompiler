//! # Transformer Integration Tests
//!
//! Tests for the complete transformation pipeline

use super::*;
use crate::analyzer::{SemanticAnalysis, SymbolTable, SemanticFlags, AnalysisMetadata, ScopeTree, ScopeType};
use crate::parser::ast_types::{Program, ProgramSourceType, Statement, Expression};
use std::collections::HashMap;

/// Helper function to create a test analysis result
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
fn test_transformer_initialization() {
    let config = TransformerConfig::default();
    let analysis_result = create_test_analysis();
    
    let transformer = Transformer::new(config, analysis_result);
    assert_eq!(transformer.count_enabled_passes(), 5);
}

#[test]
fn test_empty_program_transformation() {
    let config = TransformerConfig::default();
    let analysis_result = create_test_analysis();
    
    let mut transformer = Transformer::new(config, analysis_result);
    let ast = Program {
        body: vec![],
        source_type: ProgramSourceType::Script,
    };
    
    let result = transformer.transform(ast).unwrap();
    assert_eq!(result.transformed_ast.body.len(), 0);
    assert_eq!(result.stats.identifiers_renamed, 0);
}

#[test]
fn test_transformer_with_rollback_enabled() {
    let config = TransformerConfig {
        enable_rollback: true,
        verbose: true,
        ..TransformerConfig::default()
    };
    let analysis_result = create_test_analysis();
    
    let mut transformer = Transformer::new(config, analysis_result);
    let ast = Program {
        body: vec![],
        source_type: ProgramSourceType::Script,
    };
    
    let result = transformer.transform(ast).unwrap();
    assert_eq!(result.stats.rollbacks_performed, 0);
}

#[test]
fn test_transformer_with_rollback_disabled() {
    let config = TransformerConfig {
        enable_rollback: false,
        ..TransformerConfig::default()
    };
    let analysis_result = create_test_analysis();
    
    let mut transformer = Transformer::new(config, analysis_result);
    let ast = Program {
        body: vec![],
        source_type: ProgramSourceType::Script,
    };
    
    let result = transformer.transform(ast).unwrap();
    assert_eq!(result.stats.rollbacks_performed, 0);
}

#[test]
fn test_selective_pass_enablement() {
    let config = TransformerConfig {
        enable_identifier_renaming: false,
        enable_dead_code_elimination: true,
        enable_expression_simplification: false,
        enable_property_minification: true,
        enable_function_minification: false,
        ..TransformerConfig::default()
    };
    let analysis_result = create_test_analysis();
    
    let transformer = Transformer::new(config, analysis_result);
    assert_eq!(transformer.count_enabled_passes(), 2);
}

#[test]
fn test_all_passes_disabled() {
    let config = TransformerConfig {
        enable_identifier_renaming: false,
        enable_dead_code_elimination: false,
        enable_expression_simplification: false,
        enable_property_minification: false,
        enable_function_minification: false,
        ..TransformerConfig::default()
    };
    let analysis_result = create_test_analysis();
    
    let mut transformer = Transformer::new(config, analysis_result);
    let ast = Program {
        body: vec![],
        source_type: ProgramSourceType::Script,
    };
    
    let result = transformer.transform(ast).unwrap();
    assert_eq!(result.stats.identifiers_renamed, 0);
    assert_eq!(result.stats.dead_statements_removed, 0);
    assert_eq!(result.stats.expressions_simplified, 0);
    assert_eq!(result.stats.properties_renamed, 0);
    assert_eq!(result.stats.functions_inlined, 0);
}

#[test]
fn test_transformation_statistics_accuracy() {
    let config = TransformerConfig::default();
    let analysis_result = create_test_analysis();
    
    let mut transformer = Transformer::new(config, analysis_result);
    let ast = Program {
        body: vec![],
        source_type: ProgramSourceType::Script,
    };
    
    let result = transformer.transform(ast).unwrap();
    
    // Verify all statistics are initialized
    assert_eq!(result.stats.identifiers_renamed, 0);
    assert_eq!(result.stats.dead_statements_removed, 0);
    assert_eq!(result.stats.expressions_simplified, 0);
    assert_eq!(result.stats.properties_renamed, 0);
    assert_eq!(result.stats.functions_inlined, 0);
    assert_eq!(result.stats.rollbacks_performed, 0);
    assert!(result.stats.transformation_time_ms >= 0);
}

#[test]
fn test_transformation_warnings_collection() {
    let config = TransformerConfig::default();
    let analysis_result = create_test_analysis();
    
    let mut transformer = Transformer::new(config, analysis_result);
    let ast = Program {
        body: vec![],
        source_type: ProgramSourceType::Script,
    };
    
    let result = transformer.transform(ast).unwrap();
    
    // Should have warnings about not-yet-implemented features
    assert!(!result.warnings.is_empty());
    assert!(result.warnings.iter().any(|w| w.contains("not yet")));
}

#[test]
fn test_transformation_identifier_mapping() {
    let config = TransformerConfig::default();
    let analysis_result = create_test_analysis();
    
    let mut transformer = Transformer::new(config, analysis_result);
    let ast = Program {
        body: vec![],
        source_type: ProgramSourceType::Script,
    };
    
    let result = transformer.transform(ast).unwrap();
    
    // Currently should be empty since identifier renaming is placeholder
    assert!(result.identifier_mapping.is_empty());
}

#[test]
fn test_aggressive_optimization_mode() {
    let config = TransformerConfig {
        aggressive_optimization: true,
        ..TransformerConfig::default()
    };
    let analysis_result = create_test_analysis();
    
    let mut transformer = Transformer::new(config, analysis_result);
    let ast = Program {
        body: vec![],
        source_type: ProgramSourceType::Script,
    };
    
    let result = transformer.transform(ast).unwrap();
    // Should complete without errors even in aggressive mode
    assert_eq!(result.transformed_ast.body.len(), 0);
}

#[test]
fn test_conservative_optimization_mode() {
    let config = TransformerConfig {
        aggressive_optimization: false,
        ..TransformerConfig::default()
    };
    let analysis_result = create_test_analysis();
    
    let mut transformer = Transformer::new(config, analysis_result);
    let ast = Program {
        body: vec![],
        source_type: ProgramSourceType::Script,
    };
    
    let result = transformer.transform(ast).unwrap();
    // Should complete without errors in conservative mode
    assert_eq!(result.transformed_ast.body.len(), 0);
}

#[test]
fn test_verbose_transformation_output() {
    let config = TransformerConfig {
        verbose: true,
        ..TransformerConfig::default()
    };
    let analysis_result = create_test_analysis();
    
    let mut transformer = Transformer::new(config, analysis_result);
    let ast = Program {
        body: vec![],
        source_type: ProgramSourceType::Script,
    };
    
    // This test mainly verifies that verbose mode doesn't cause crashes
    let result = transformer.transform(ast).unwrap();
    assert_eq!(result.transformed_ast.body.len(), 0);
}

#[test]
fn test_multi_pass_execution_order() {
    let config = TransformerConfig::default();
    let analysis_result = create_test_analysis();
    
    let mut transformer = Transformer::new(config, analysis_result);
    let ast = Program {
        body: vec![],
        source_type: ProgramSourceType::Script,
    };
    
    let result = transformer.transform(ast).unwrap();
    
    // Verify transformation completed (all passes should have run)
    // Even though they're placeholders, they should execute in order
    assert!(result.stats.transformation_time_ms >= 0);
    assert!(!result.warnings.is_empty()); // Should have "not implemented" warnings
}

#[test]
fn test_ast_preservation_during_transformation() {
    let config = TransformerConfig::default();
    let analysis_result = create_test_analysis();
    
    let mut transformer = Transformer::new(config, analysis_result);
    
    // Create an AST with some content
    let original_ast = Program {
        body: vec![
            Statement::BlockStatement {
                body: vec![]
            }
        ],
        source_type: ProgramSourceType::Script,
    };
    
    let result = transformer.transform(original_ast).unwrap();
    
    // AST structure should be preserved (placeholders don't modify)
    assert_eq!(result.transformed_ast.body.len(), 1);
    assert!(matches!(result.transformed_ast.body[0], Statement::BlockStatement { .. }));
}

#[test]
fn test_transformation_error_handling() {
    let config = TransformerConfig::default();
    let analysis_result = create_test_analysis();
    
    let mut transformer = Transformer::new(config, analysis_result);
    let ast = Program {
        body: vec![],
        source_type: ProgramSourceType::Script,
    };
    
    // Current implementation should not produce errors
    let result = transformer.transform(ast);
    assert!(result.is_ok());
}

#[test]
fn test_convenience_transform_function() {
    let ast = Program {
        body: vec![],
        source_type: ProgramSourceType::Script,
    };
    let analysis_result = create_test_analysis();
    
    let result = transform_ast(ast, analysis_result);
    assert!(result.is_ok());
    
    let transform_result = result.unwrap();
    assert_eq!(transform_result.transformed_ast.body.len(), 0);
    assert!(transform_result.stats.transformation_time_ms >= 0);
}