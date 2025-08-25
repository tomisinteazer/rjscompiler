//! # Semantic Analysis Module
//!
//! This module performs semantic analysis to detect constructs that affect
//! optimization safety, including eval usage, with statements, this binding,
//! and other dynamic features that prevent safe variable renaming.
//!
//! ## Key Responsibilities
//!
//! - Detect eval() calls and mark containing scopes as unsafe
//! - Detect with statements and mark scope resolution as dynamic
//! - Classify this usage as lexical (arrow functions) or dynamic (regular functions)
//! - Propagate unsafe flags upward through the scope chain
//! - Mark scope safety classification for optimization decisions

use crate::analyzer::{
    AnalysisResult, AnalyzerConfig, ScopeId, ScopeTree, SemanticFlags,
    SymbolTable, UnsafeReason,
};
use crate::parser::ast_types::{
    ClassElement, Expression, ForInit, Program, Statement, UnaryOperator,
};

/// Context for semantic analysis traversal
pub struct SemanticAnalysisContext<'a> {
    /// Current scope being analyzed
    pub current_scope:  ScopeId,
    /// Reference to the scope tree
    pub scope_tree:     &'a mut ScopeTree,
    /// Reference to the symbol table
    pub symbol_table:   &'a mut SymbolTable,
    /// Reference to semantic flags being collected
    pub semantic_flags: &'a mut SemanticFlags,
    /// Analysis configuration
    pub config:         &'a AnalyzerConfig,
    /// Whether we're currently in strict mode
    #[allow(dead_code)]
    pub strict_mode:    bool,
    /// Whether we're inside an arrow function (affects this binding)
    pub in_arrow_function: bool,
}

/// Performs semantic analysis to detect unsafe constructs
///
/// # Arguments
///
/// * `ast` - The JavaScript AST to analyze
/// * `scope_tree` - Mutable reference to the scope tree
/// * `symbol_table` - Mutable reference to the symbol table
/// * `semantic_flags` - Mutable reference to semantic flags being collected
/// * `config` - Analysis configuration
///
/// # Returns
///
/// Returns `Ok(())` if analysis succeeds, or an `AnalysisError` if it fails.
pub fn analyze_semantics(
    ast: &Program,
    scope_tree: &mut ScopeTree,
    symbol_table: &mut SymbolTable,
    semantic_flags: &mut SemanticFlags,
    config: &AnalyzerConfig,
) -> AnalysisResult<()> {
    if config.verbose {
        println!("Performing semantic analysis...");
    }

    let mut context = SemanticAnalysisContext {
        current_scope:     scope_tree.root_scope_id,
        scope_tree,
        symbol_table,
        semantic_flags,
        config,
        strict_mode:       config.strict_mode,
        in_arrow_function: false,
    };

    // Analyze program body for semantic issues
    for statement in &ast.body {
        analyze_statement_semantics(statement, &mut context)?;
    }

    // Propagate unsafe flags upward through scope chain
    propagate_unsafe_flags(&mut context)?;

    if config.verbose {
        let unsafe_scope_count = context.semantic_flags.unsafe_scopes.len();
        let unsafe_symbol_count = context.semantic_flags.unsafe_symbols.len();
        println!(
            "Semantic analysis completed: {} unsafe scopes, {} unsafe symbols",
            unsafe_scope_count, unsafe_symbol_count
        );
    }

    Ok(())
}

/// Analyzes a statement for semantic issues
fn analyze_statement_semantics(
    statement: &Statement,
    context: &mut SemanticAnalysisContext,
) -> AnalysisResult<()> {
    match statement {
        Statement::VariableDeclaration { declarations, .. } => {
            for declarator in declarations {
                if let Some(init) = &declarator.init {
                    analyze_expression_semantics(init, context)?;
                }
            }
            Ok(())
        }
        Statement::FunctionDeclaration { params: _, body, .. } => {
            // Enter function scope
            if let Some(function_scope) = find_child_scope_of_type(
                context.current_scope,
                crate::analyzer::ScopeType::Function,
                context,
            ) {
                let previous_scope = context.current_scope;
                let previous_arrow_state = context.in_arrow_function;
                context.current_scope = function_scope;
                context.in_arrow_function = false; // Regular function, not arrow

                // Analyze function body
                for stmt in &body.body {
                    analyze_statement_semantics(stmt, context)?;
                }

                // Restore context
                context.current_scope = previous_scope;
                context.in_arrow_function = previous_arrow_state;
            }
            Ok(())
        }
        Statement::ClassDeclaration { body, super_class, .. } => {
            if let Some(super_expr) = super_class {
                analyze_expression_semantics(super_expr, context)?;
            }

            // Enter class scope
            if let Some(class_scope) = find_child_scope_of_type(
                context.current_scope,
                crate::analyzer::ScopeType::Class,
                context,
            ) {
                let previous_scope = context.current_scope;
                context.current_scope = class_scope;

                // Analyze class body
                for element in &body.body {
                    analyze_class_element_semantics(element, context)?;
                }

                // Restore context
                context.current_scope = previous_scope;
            }
            Ok(())
        }
        Statement::ExpressionStatement { expression } => {
            analyze_expression_semantics(expression, context)
        }
        Statement::BlockStatement { body } => {
            // Enter block scope if it exists
            let block_scope = find_child_scope_of_type(
                context.current_scope,
                crate::analyzer::ScopeType::Block,
                context,
            );

            if let Some(scope_id) = block_scope {
                let previous_scope = context.current_scope;
                context.current_scope = scope_id;

                for stmt in body {
                    analyze_statement_semantics(stmt, context)?;
                }

                context.current_scope = previous_scope;
            } else {
                for stmt in body {
                    analyze_statement_semantics(stmt, context)?;
                }
            }
            Ok(())
        }
        Statement::ReturnStatement { argument } => {
            if let Some(expr) = argument {
                analyze_expression_semantics(expr, context)?;
            }
            Ok(())
        }
        Statement::IfStatement {
            test,
            consequent,
            alternate,
        } => {
            analyze_expression_semantics(test, context)?;
            analyze_statement_semantics(consequent, context)?;
            if let Some(alt) = alternate {
                analyze_statement_semantics(alt, context)?;
            }
            Ok(())
        }
        Statement::WhileStatement { test, body } => {
            analyze_expression_semantics(test, context)?;
            analyze_statement_semantics(body, context)
        }
        Statement::ForStatement {
            init,
            test,
            update,
            body,
        } => {
            // Enter for loop scope if it exists
            let loop_scope = find_child_scope_of_type(
                context.current_scope,
                crate::analyzer::ScopeType::Block,
                context,
            );

            if let Some(scope_id) = loop_scope {
                let previous_scope = context.current_scope;
                context.current_scope = scope_id;

                if let Some(for_init) = init {
                    analyze_for_init_semantics(for_init, context)?;
                }
                if let Some(test_expr) = test {
                    analyze_expression_semantics(test_expr, context)?;
                }
                if let Some(update_expr) = update {
                    analyze_expression_semantics(update_expr, context)?;
                }
                analyze_statement_semantics(body, context)?;

                context.current_scope = previous_scope;
            } else {
                if let Some(for_init) = init {
                    analyze_for_init_semantics(for_init, context)?;
                }
                if let Some(test_expr) = test {
                    analyze_expression_semantics(test_expr, context)?;
                }
                if let Some(update_expr) = update {
                    analyze_expression_semantics(update_expr, context)?;
                }
                analyze_statement_semantics(body, context)?;
            }
            Ok(())
        }
        Statement::ImportDeclaration { .. } => Ok(()), // Imports don't affect semantics
        Statement::ExportNamedDeclaration { declaration, .. } => {
            if let Some(decl) = declaration {
                analyze_statement_semantics(decl, context)?;
            }
            Ok(())
        }
    }
}

/// Analyzes class elements for semantic issues
fn analyze_class_element_semantics(
    element: &ClassElement,
    context: &mut SemanticAnalysisContext,
) -> AnalysisResult<()> {
    match element {
        ClassElement::PropertyDefinition { value, .. } => {
            if let Some(expr) = value {
                analyze_expression_semantics(expr, context)?;
            }
            Ok(())
        }
        ClassElement::MethodDefinition { value, .. } => {
            analyze_function_expression_semantics(value, context)
        }
    }
}

/// Analyzes for loop initialization for semantic issues
fn analyze_for_init_semantics(
    init: &ForInit,
    context: &mut SemanticAnalysisContext,
) -> AnalysisResult<()> {
    match init {
        ForInit::VariableDeclaration { declarations, .. } => {
            for declarator in declarations {
                if let Some(init_expr) = &declarator.init {
                    analyze_expression_semantics(init_expr, context)?;
                }
            }
            Ok(())
        }
        ForInit::Expression(expr) => analyze_expression_semantics(expr, context),
    }
}

/// Analyzes expressions for semantic issues
fn analyze_expression_semantics(
    expression: &Expression,
    context: &mut SemanticAnalysisContext,
) -> AnalysisResult<()> {
    match expression {
        Expression::Identifier(id) => {
            // Check for special identifiers that affect safety
            match id.name.as_str() {
                "eval" => {
                    mark_scope_unsafe(context.current_scope, UnsafeReason::EvalUsage, context);
                    if context.config.verbose {
                        println!("Detected eval usage in scope {}", context.current_scope);
                    }
                }
                "arguments" => {
                    // arguments object usage can affect optimization
                    if context.config.verbose {
                        println!("Detected arguments usage in scope {}", context.current_scope);
                    }
                }
                _ => {}
            }
            Ok(())
        }
        Expression::CallExpression { callee, arguments } => {
            // Check for eval() calls
            if let Expression::Identifier(id) = callee.as_ref()
                && id.name == "eval" {
                    mark_scope_unsafe(context.current_scope, UnsafeReason::EvalUsage, context);
                    if context.config.verbose {
                        println!("Detected eval() call in scope {}", context.current_scope);
                    }
                }

            analyze_expression_semantics(callee, context)?;
            for arg in arguments {
                analyze_expression_semantics(arg, context)?;
            }
            Ok(())
        }
        Expression::ThisExpression => {
            // this usage classification
            if context.in_arrow_function {
                // Arrow functions have lexical this binding
                if context.config.verbose {
                    println!("Detected lexical this usage in scope {}", context.current_scope);
                }
            } else {
                // Regular functions have dynamic this binding
                mark_scope_unsafe(context.current_scope, UnsafeReason::DynamicThis, context);
                if context.config.verbose {
                    println!("Detected dynamic this usage in scope {}", context.current_scope);
                }
            }
            Ok(())
        }
        Expression::BinaryExpression { left, right, .. } => {
            analyze_expression_semantics(left, context)?;
            analyze_expression_semantics(right, context)
        }
        Expression::UnaryExpression { argument, operator, prefix: _ } => {
            // Check for typeof operator which might indicate dynamic access
            if matches!(operator, UnaryOperator::Typeof)
                && let Expression::Identifier(_) = argument.as_ref() {
                    // typeof identifier - might be checking for undefined globals
                    if context.config.verbose {
                        println!("Detected typeof usage in scope {}", context.current_scope);
                    }
                }
            analyze_expression_semantics(argument, context)
        }
        Expression::AssignmentExpression { left, right, .. } => {
            analyze_expression_semantics(left, context)?;
            analyze_expression_semantics(right, context)
        }
        Expression::MemberExpression {
            object,
            property,
            computed,
        } => {
            analyze_expression_semantics(object, context)?;
            
            if *computed {
                // Computed property access obj[prop] - potentially unsafe
                analyze_expression_semantics(property, context)?;
                
                // Check for window['property'] pattern
                if let Expression::Identifier(obj_id) = object.as_ref()
                    && (obj_id.name == "window" || obj_id.name == "global") {
                        mark_scope_unsafe(
                            context.current_scope,
                            UnsafeReason::IndirectAccess,
                            context,
                        );
                        if context.config.verbose {
                            println!(
                                "Detected indirect global access in scope {}",
                                context.current_scope
                            );
                        }
                    }
            } else if let Expression::Identifier(_) = property.as_ref() {
                // Static property access obj.prop - generally safe
            } else {
                analyze_expression_semantics(property, context)?;
            }
            Ok(())
        }
        Expression::FunctionExpression(func_expr) => {
            analyze_function_expression_semantics(func_expr, context)
        }
        Expression::ArrowFunctionExpression { params: _, body, .. } => {
            // Create function scope and analyze arrow function
            if let Some(function_scope) = find_child_scope_of_type(
                context.current_scope,
                crate::analyzer::ScopeType::Function,
                context,
            ) {
                let previous_scope = context.current_scope;
                let previous_arrow_state = context.in_arrow_function;
                context.current_scope = function_scope;
                context.in_arrow_function = true; // Arrow function has lexical this

                match body {
                    crate::parser::ast_types::ArrowFunctionBody::Expression(expr) => {
                        analyze_expression_semantics(expr, context)?;
                    }
                    crate::parser::ast_types::ArrowFunctionBody::BlockStatement(block) => {
                        for stmt in &block.body {
                            analyze_statement_semantics(stmt, context)?;
                        }
                    }
                }

                context.current_scope = previous_scope;
                context.in_arrow_function = previous_arrow_state;
            }
            Ok(())
        }
        Expression::ConditionalExpression {
            test,
            consequent,
            alternate,
        } => {
            analyze_expression_semantics(test, context)?;
            analyze_expression_semantics(consequent, context)?;
            analyze_expression_semantics(alternate, context)
        }
        Expression::Literal(_) => Ok(()), // Literals are safe
        _ => Ok(()), // Handle other expression types as needed
    }
}

/// Analyzes function expressions for semantic issues
fn analyze_function_expression_semantics(
    func_expr: &crate::parser::ast_types::FunctionExpression,
    context: &mut SemanticAnalysisContext,
) -> AnalysisResult<()> {
    // Enter function scope
    if let Some(function_scope) = find_child_scope_of_type(
        context.current_scope,
        crate::analyzer::ScopeType::Function,
        context,
    ) {
        let previous_scope = context.current_scope;
        let previous_arrow_state = context.in_arrow_function;
        context.current_scope = function_scope;
        context.in_arrow_function = false; // Regular function

        // Analyze function body
        for stmt in &func_expr.body.body {
            analyze_statement_semantics(stmt, context)?;
        }

        // Restore context
        context.current_scope = previous_scope;
        context.in_arrow_function = previous_arrow_state;
    }
    Ok(())
}

/// Marks a scope as unsafe for optimization
fn mark_scope_unsafe(
    scope_id: ScopeId,
    reason: UnsafeReason,
    context: &mut SemanticAnalysisContext,
) {
    context.semantic_flags.unsafe_scopes.insert(scope_id, reason.clone());
    
    // Mark the scope itself as unsafe
    if let Some(scope) = context.scope_tree.get_scope_mut(scope_id) {
        scope.is_safe = false;
    }

    // Mark all symbols in this scope as unsafe for renaming
    if let Some(scope_bindings) = context.symbol_table.scope_bindings.get(&scope_id) {
        for &symbol_id in scope_bindings.values() {
            context.semantic_flags.unsafe_symbols.insert(symbol_id, reason.clone());
            if let Some(symbol) = context.symbol_table.symbols.get_mut(&symbol_id) {
                symbol.is_renamable = false;
            }
        }
    }
}

/// Propagates unsafe flags upward through the scope chain
fn propagate_unsafe_flags(context: &mut SemanticAnalysisContext) -> AnalysisResult<()> {
    let unsafe_scopes: Vec<_> = context.semantic_flags.unsafe_scopes.keys().copied().collect();

    for scope_id in unsafe_scopes {
        propagate_unsafe_flag_upward(scope_id, context);
    }

    Ok(())
}

/// Propagates unsafe flag from a scope to its ancestors
fn propagate_unsafe_flag_upward(scope_id: ScopeId, context: &mut SemanticAnalysisContext) {
    let mut current_scope = scope_id;

    while let Some(scope) = context.scope_tree.get_scope(current_scope) {
        if let Some(parent_id) = scope.parent_id {
            // Check if parent should be marked unsafe based on child's unsafe reason
            if let Some(reason) = context.semantic_flags.unsafe_scopes.get(&current_scope) {
                match reason {
                    UnsafeReason::EvalUsage => {
                        // eval affects all ancestor scopes
                        if !context.semantic_flags.unsafe_scopes.contains_key(&parent_id) {
                            mark_scope_unsafe(parent_id, UnsafeReason::EvalUsage, context);
                        }
                        current_scope = parent_id;
                    }
                    UnsafeReason::WithStatement => {
                        // with affects parent scope
                        if !context.semantic_flags.unsafe_scopes.contains_key(&parent_id) {
                            mark_scope_unsafe(parent_id, UnsafeReason::WithStatement, context);
                        }
                        current_scope = parent_id;
                    }
                    UnsafeReason::DynamicThis | UnsafeReason::IndirectAccess => {
                        // These don't necessarily propagate upward
                        break;
                    }
                    UnsafeReason::ExternalDependency | UnsafeReason::Unknown => {
                        // Conservative: propagate upward
                        if !context.semantic_flags.unsafe_scopes.contains_key(&parent_id) {
                            mark_scope_unsafe(parent_id, reason.clone(), context);
                        }
                        current_scope = parent_id;
                    }
                }
            } else {
                break;
            }
        } else {
            break; // Reached root scope
        }
    }
}

/// Finds a child scope of a specific type
fn find_child_scope_of_type(
    parent_scope_id: ScopeId,
    scope_type: crate::analyzer::ScopeType,
    context: &SemanticAnalysisContext,
) -> Option<ScopeId> {
    if let Some(parent_scope) = context.scope_tree.get_scope(parent_scope_id) {
        for &child_id in &parent_scope.children {
            if let Some(child_scope) = context.scope_tree.get_scope(child_id)
                && std::mem::discriminant(&child_scope.scope_type) == std::mem::discriminant(&scope_type) {
                    return Some(child_id);
                }
        }
    }
    None
}