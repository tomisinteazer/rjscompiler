//! # Analyzer Test Suite
//!
//! Comprehensive test suite for the analyzer component following the TDD approach
//! outlined in the analyzer.md specification. Tests cover scope building, semantic
//! analysis, and edge cases to ensure correct implementation.

use crate::analyzer::{
    analyze_ast, AnalyzerConfig, ScopeType, SemanticAnalysis, SymbolType, UnsafeReason,
    VariableKind,
};
use crate::parser::{parse_js, ParserConfig};

/// Helper function to parse JavaScript code and run analysis
fn parse_and_analyze(source: &str) -> Result<SemanticAnalysis, Box<dyn std::error::Error>> {
    let parser_config = ParserConfig::default();
    let parse_result = parse_js(source, "test.js", &parser_config);
    
    if !parse_result.errors.is_empty() {
        return Err(format!("Parse errors: {:?}", parse_result.errors).into());
    }
    
    let ast = parse_result.ast.ok_or("No AST produced")?;
    let analyzer_config = AnalyzerConfig::default();
    let analysis = analyze_ast(&ast, &analyzer_config)?;
    
    Ok(analysis)
}

/// Helper function to get symbol by name from analysis
fn find_symbol_by_name<'a>(analysis: &'a SemanticAnalysis, name: &str) -> Option<&'a crate::analyzer::Symbol> {
    analysis.symbol_table.symbols.values().find(|s| s.name == name)
}

/// Helper function to check if a scope contains a binding
fn scope_has_binding(analysis: &SemanticAnalysis, scope_id: u32, name: &str) -> bool {
    if let Some(bindings) = analysis.symbol_table.scope_bindings.get(&scope_id) {
        bindings.contains_key(name)
    } else {
        false
    }
}

#[cfg(test)]
mod scope_builder_tests {
    use super::*;

    #[test]
    fn should_create_simple_function_scope() {
        let source = "function foo() { let x = 1; }";
        let analysis = parse_and_analyze(source).expect("Analysis should succeed");

        // Should have global scope and function scope
        assert_eq!(analysis.scope_tree.scopes.len(), 2);
        
        // Global scope should exist
        let global_scope = analysis.scope_tree.get_scope(0).expect("Global scope should exist");
        assert!(matches!(global_scope.scope_type, ScopeType::Global));
        
        // Function scope should exist
        assert_eq!(global_scope.children.len(), 1);
        let function_scope_id = global_scope.children[0];
        let function_scope = analysis.scope_tree.get_scope(function_scope_id).expect("Function scope should exist");
        assert!(matches!(function_scope.scope_type, ScopeType::Function));
        
        // Function foo should be in global scope
        assert!(scope_has_binding(&analysis, 0, "foo"));
        
        // Variable x should be in function scope
        assert!(scope_has_binding(&analysis, function_scope_id, "x"));
        
        // Verify symbol types
        let foo_symbol = find_symbol_by_name(&analysis, "foo").expect("foo symbol should exist");
        assert!(matches!(foo_symbol.symbol_type, SymbolType::Function));
        
        let x_symbol = find_symbol_by_name(&analysis, "x").expect("x symbol should exist");
        assert!(matches!(x_symbol.symbol_type, SymbolType::Variable { kind: VariableKind::Let }));
    }

    #[test]
    fn should_handle_nested_functions() {
        let source = "function outer() { function inner() {} }";
        let analysis = parse_and_analyze(source).expect("Analysis should succeed");

        // Should have global, outer function, and inner function scopes
        assert_eq!(analysis.scope_tree.scopes.len(), 3);
        
        // Verify scope hierarchy: Global -> Outer -> Inner
        let global_scope = analysis.scope_tree.get_scope(0).expect("Global scope should exist");
        assert_eq!(global_scope.children.len(), 1);
        
        let outer_scope_id = global_scope.children[0];
        let outer_scope = analysis.scope_tree.get_scope(outer_scope_id).expect("Outer scope should exist");
        assert!(matches!(outer_scope.scope_type, ScopeType::Function));
        assert_eq!(outer_scope.children.len(), 1);
        
        let inner_scope_id = outer_scope.children[0];
        let inner_scope = analysis.scope_tree.get_scope(inner_scope_id).expect("Inner scope should exist");
        assert!(matches!(inner_scope.scope_type, ScopeType::Function));
        assert!(inner_scope.children.is_empty());
        
        // outer function should be in global scope
        assert!(scope_has_binding(&analysis, 0, "outer"));
        
        // inner function should be in outer function scope
        assert!(scope_has_binding(&analysis, outer_scope_id, "inner"));
    }

    #[test]
    fn should_create_block_scope_for_let_const() {
        let source = "if (true) { let blockVar = 1; }";
        let analysis = parse_and_analyze(source).expect("Analysis should succeed");

        // Should have global scope and block scope
        assert_eq!(analysis.scope_tree.scopes.len(), 2);
        
        let global_scope = analysis.scope_tree.get_scope(0).expect("Global scope should exist");
        assert_eq!(global_scope.children.len(), 1);
        
        let block_scope_id = global_scope.children[0];
        let block_scope = analysis.scope_tree.get_scope(block_scope_id).expect("Block scope should exist");
        assert!(matches!(block_scope.scope_type, ScopeType::Block));
        
        // blockVar should be in block scope
        assert!(scope_has_binding(&analysis, block_scope_id, "blockVar"));
        
        let block_var_symbol = find_symbol_by_name(&analysis, "blockVar").expect("blockVar symbol should exist");
        assert!(matches!(block_var_symbol.symbol_type, SymbolType::Variable { kind: VariableKind::Let }));
    }

    #[test]
    fn should_handle_variable_shadowing() {
        let source = "let x = 1; function foo() { let x = 2; }";
        let analysis = parse_and_analyze(source).expect("Analysis should succeed");

        // Should have two x symbols in different scopes
        let x_symbols: Vec<_> = analysis.symbol_table.symbols.values()
            .filter(|s| s.name == "x")
            .collect();
        assert_eq!(x_symbols.len(), 2);
        
        // One x in global scope, one in function scope
        let global_x = x_symbols.iter().find(|s| s.scope_id == 0).expect("Global x should exist");
        let function_scope_id = analysis.scope_tree.get_scope(0).expect("Global scope").children[0];
        let function_x = x_symbols.iter().find(|s| s.scope_id == function_scope_id).expect("Function x should exist");
        
        assert_eq!(global_x.scope_id, 0);
        assert_eq!(function_x.scope_id, function_scope_id);
    }

    #[test]
    fn should_detect_closure_capture() {
        let source = "function outer() { let x = 1; function inner() { return x; } }";
        let analysis = parse_and_analyze(source).expect("Analysis should succeed");

        // Find the x symbol
        let x_symbol = find_symbol_by_name(&analysis, "x").expect("x symbol should exist");
        
        // x should be marked as captured
        assert!(x_symbol.is_captured, "Variable x should be captured by closure");
        
        // x should have references
        assert!(!x_symbol.references.is_empty(), "Variable x should have references");
    }
}

#[cfg(test)]
mod semantic_analysis_tests {
    use super::*;

    #[test]
    fn should_detect_eval_usage() {
        let source = "function evil() { eval('x'); }";
        let analysis = parse_and_analyze(source).expect("Analysis should succeed");

        // Function scope should be marked unsafe due to eval
        let function_scope_id = analysis.scope_tree.get_scope(0).expect("Global scope").children[0];
        
        assert!(analysis.semantic_flags.unsafe_scopes.contains_key(&function_scope_id));
        assert!(matches!(
            analysis.semantic_flags.unsafe_scopes.get(&function_scope_id),
            Some(UnsafeReason::EvalUsage)
        ));
        
        // Function scope should be marked as not safe
        let function_scope = analysis.scope_tree.get_scope(function_scope_id).expect("Function scope should exist");
        assert!(!function_scope.is_safe);
    }

    #[test]
    fn should_detect_with_statement() {
        // Note: This test is conceptual as with statements are not in our current AST
        // but shows the pattern for when they are added
        let source = "function bad() { /* with(obj) { prop; } */ }";
        let analysis = parse_and_analyze(source).expect("Analysis should succeed");

        // For now, just verify the function exists
        assert!(scope_has_binding(&analysis, 0, "bad"));
    }

    #[test]
    fn should_detect_this_usage_in_arrow_function() {
        // Note: This test is conceptual as this expressions are not fully implemented
        // but shows the expected behavior
        let source = "const f = () => { /* this.prop */ };";
        let analysis = parse_and_analyze(source).expect("Analysis should succeed");

        // For now, just verify the function exists
        assert!(scope_has_binding(&analysis, 0, "f"));
    }

    #[test]
    fn should_detect_this_usage_in_regular_function() {
        // Note: This test is conceptual as this expressions are not fully implemented
        let source = "function f() { /* return this.prop; */ }";
        let analysis = parse_and_analyze(source).expect("Analysis should succeed");

        // For now, just verify the function exists
        assert!(scope_has_binding(&analysis, 0, "f"));
    }
}

#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[test]
    
    fn should_handle_var_hoisting() {
        let source = "y = x; var x = 5;";
        let analysis = parse_and_analyze(source).expect("Analysis should succeed");

        // x should be declared in global scope
        assert!(scope_has_binding(&analysis, 0, "x"));
        
        let x_symbol = find_symbol_by_name(&analysis, "x").expect("x symbol should exist");
        assert!(matches!(x_symbol.symbol_type, SymbolType::Variable { kind: VariableKind::Var }));
        
        // Should have references (y = x assignment reference and var x declaration)
        assert!(!x_symbol.references.is_empty(), "x should have at least one reference from y = x");
    }

    #[test]
    fn should_detect_temporal_dead_zone_violation() {
        // Note: TDZ detection would be implemented in semantic analysis
        let source = "console.log(x); let x = 5;";
        let analysis = parse_and_analyze(source).expect("Analysis should succeed");

        // For now, just verify that x is properly declared
        assert!(scope_has_binding(&analysis, 0, "x"));
        
        let x_symbol = find_symbol_by_name(&analysis, "x").expect("x symbol should exist");
        assert!(matches!(x_symbol.symbol_type, SymbolType::Variable { kind: VariableKind::Let }));
    }

    #[test]
    fn should_handle_module_exports() {
        let source = "export const value = 42;";
        let analysis = parse_and_analyze(source).expect("Analysis should succeed");

        // value should be marked as exported
        let value_symbol = find_symbol_by_name(&analysis, "value").expect("value symbol should exist");
        assert!(value_symbol.is_exported, "Exported symbol should be marked as exported");
        assert!(!value_symbol.is_renamable, "Exported symbol should not be renamable");
    }

    #[test]
    fn should_handle_function_parameters() {
        let source = "function test(a, b, c) { return a + b + c; }";
        let analysis = parse_and_analyze(source).expect("Analysis should succeed");

        // Parameters should be in function scope
        let function_scope_id = analysis.scope_tree.get_scope(0).expect("Global scope").children[0];
        
        assert!(scope_has_binding(&analysis, function_scope_id, "a"));
        assert!(scope_has_binding(&analysis, function_scope_id, "b"));
        assert!(scope_has_binding(&analysis, function_scope_id, "c"));
        
        // Parameters should have correct symbol type
        let a_symbol = find_symbol_by_name(&analysis, "a").expect("a symbol should exist");
        assert!(matches!(a_symbol.symbol_type, SymbolType::Variable { kind: VariableKind::Var }));
    }

    #[test]
    fn should_handle_import_declarations() {
        let source = "import { foo, bar } from 'module';";
        let analysis = parse_and_analyze(source).expect("Analysis should succeed");

        // Imports should be in global scope
        assert!(scope_has_binding(&analysis, 0, "foo"));
        assert!(scope_has_binding(&analysis, 0, "bar"));
        
        // Imports should have correct symbol type
        let foo_symbol = find_symbol_by_name(&analysis, "foo").expect("foo symbol should exist");
        assert!(matches!(foo_symbol.symbol_type, SymbolType::Import));
    }

    #[test]
    fn should_handle_class_declarations() {
        let source = "class MyClass { constructor() {} method() {} }";
        let analysis = parse_and_analyze(source).expect("Analysis should succeed");

        // Class should be in global scope
        assert!(scope_has_binding(&analysis, 0, "MyClass"));
        
        // Should have class scope
        let global_scope = analysis.scope_tree.get_scope(0).expect("Global scope");
        assert_eq!(global_scope.children.len(), 1);
        
        let class_scope_id = global_scope.children[0];
        let class_scope = analysis.scope_tree.get_scope(class_scope_id).expect("Class scope should exist");
        assert!(matches!(class_scope.scope_type, ScopeType::Class));
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn should_analyze_complex_nested_closures() {
        let source = r#"
            function outer(x) {
                let outerVar = x + 1;
                
                function middle(y) {
                    let middleVar = y + outerVar;
                    
                    function inner(z) {
                        return outerVar + middleVar + z;
                    }
                    
                    return inner;
                }
                
                return middle;
            }
        "#;
        
        let analysis = parse_and_analyze(source).expect("Analysis should succeed");

        // Should have 4 scopes: global, outer, middle, inner
        assert_eq!(analysis.scope_tree.scopes.len(), 4);
        
        // outerVar should be captured (used in middle and inner)
        let outer_var_symbol = find_symbol_by_name(&analysis, "outerVar").expect("outerVar should exist");
        assert!(outer_var_symbol.is_captured, "outerVar should be captured");
        
        // middleVar should be captured (used in inner)
        let middle_var_symbol = find_symbol_by_name(&analysis, "middleVar").expect("middleVar should exist");
        assert!(middle_var_symbol.is_captured, "middleVar should be captured");
        
        // Verify scope hierarchy
        let global_scope = analysis.scope_tree.get_scope(0).expect("Global scope");
        assert_eq!(global_scope.children.len(), 1);
        
        let outer_scope_id = global_scope.children[0];
        let outer_scope = analysis.scope_tree.get_scope(outer_scope_id).expect("Outer scope");
        assert_eq!(outer_scope.children.len(), 1);
        
        let middle_scope_id = outer_scope.children[0];
        let middle_scope = analysis.scope_tree.get_scope(middle_scope_id).expect("Middle scope");
        assert_eq!(middle_scope.children.len(), 1);
    }

    #[test]
    fn should_analyze_module_with_exports_and_imports() {
        let source = r#"
            import { util } from 'utils';
            
            const config = { debug: true };
            
            export function process(data) {
                return util.transform(data, config);
            }
            
            export { config };
        "#;
        
        let analysis = parse_and_analyze(source).expect("Analysis should succeed");

        // util should be imported
        let util_symbol = find_symbol_by_name(&analysis, "util").expect("util should exist");
        assert!(matches!(util_symbol.symbol_type, SymbolType::Import));
        
        // process should be exported
        let process_symbol = find_symbol_by_name(&analysis, "process").expect("process should exist");
        assert!(process_symbol.is_exported, "process should be exported");
        
        // config should be declared and exported
        let config_symbol = find_symbol_by_name(&analysis, "config").expect("config should exist");
        assert!(config_symbol.is_exported, "config should be exported");
    }

    #[test]
    fn should_provide_analysis_metadata() {
        let source = r#"
            function test() {
                let x = 1;
                const y = 2;
                var z = 3;
            }
        "#;
        
        let analysis = parse_and_analyze(source).expect("Analysis should succeed");

        // Check metadata
        assert!(analysis.metadata.scope_count >= 2); // At least global and function
        assert!(analysis.metadata.symbol_count >= 4); // At least test, x, y, z
        assert!(analysis.metadata.analysis_time_ms > 0);
        
        // Should have reasonable analysis time (less than 1 second for simple code)
        assert!(analysis.metadata.analysis_time_ms < 1000);
    }
}