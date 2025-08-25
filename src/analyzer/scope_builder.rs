//! # Scope Builder Module
//!
//! This module is responsible for constructing the hierarchical scope tree and
//! tracking symbol bindings throughout the JavaScript AST. It handles all types
//! of scopes including global, function, block, class, and module scopes.
//!
//! ## Key Responsibilities
//!
//! - Construct scope tree with unique scope IDs and parent-child relationships
//! - Enter new scope at functions, blocks, catch clauses, classes, modules
//! - Maintain symbol bindings for variables, functions, classes, parameters
//! - Track references to identifiers (read/write/declare)
//! - Resolve shadowing and redeclarations correctly
//! - Detect captures: mark when an inner scope closes over an outer variable


use crate::analyzer::{
    AnalysisResult, AnalyzerConfig, ReferenceType, Scope, ScopeId, ScopeTree,
    ScopeType, SemanticFlags, SourceLocation, Symbol, SymbolId, SymbolReference, SymbolTable,
    SymbolType, VariableKind,
};
use crate::parser::ast_types::{
    ClassElement, Expression, ForInit, Identifier, ImportSpecifier, Pattern, Program, Statement,
    VariableDeclarationKind,
};

/// Context for scope analysis traversal
pub struct ScopeAnalysisContext<'a> {
    /// Current scope being analyzed
    pub current_scope:  ScopeId,
    /// Reference to the scope tree
    pub scope_tree:     &'a mut ScopeTree,
    /// Reference to the symbol table
    pub symbol_table:   &'a mut SymbolTable,
    /// Reference to semantic flags
    #[allow(dead_code)]
    pub semantic_flags: &'a mut SemanticFlags,
    /// Analysis configuration
    pub config:         &'a AnalyzerConfig,
    /// Current source location (for error reporting)
    pub current_location: SourceLocation,
}

/// Analyzes scopes throughout the AST and builds the scope tree
///
/// # Arguments
///
/// * `ast` - The JavaScript AST to analyze
/// * `scope_tree` - Mutable reference to the scope tree being built
/// * `symbol_table` - Mutable reference to the symbol table being populated
/// * `semantic_flags` - Mutable reference to semantic flags being collected
/// * `config` - Analysis configuration
///
/// # Returns
///
/// Returns `Ok(())` if analysis succeeds, or an `AnalysisError` if it fails.
pub fn analyze_scopes(
    ast: &Program,
    scope_tree: &mut ScopeTree,
    symbol_table: &mut SymbolTable,
    semantic_flags: &mut SemanticFlags,
    config: &AnalyzerConfig,
) -> AnalysisResult<()> {
    if config.verbose {
        println!("Building scope tree...");
    }

    let mut context = ScopeAnalysisContext {
        current_scope:    scope_tree.root_scope_id,
        scope_tree,
        symbol_table,
        semantic_flags,
        config,
        current_location: SourceLocation {
            line:   1,
            column: 0,
            offset: 0,
        },
    };

    // PHASE 1: Hoist var declarations and function declarations
    // This implements JavaScript's hoisting behavior
    hoist_declarations(ast, &mut context)?;

    // PHASE 2: Analyze program body normally
    for statement in &ast.body {
        analyze_statement(statement, &mut context)?;
    }

    if config.verbose {
        println!(
            "Scope tree built: {} scopes, {} symbols",
            context.scope_tree.next_scope_id,
            context.symbol_table.next_symbol_id
        );
    }

    Ok(())
}

/// Implements JavaScript hoisting by pre-declaring var variables and function declarations
/// This ensures that var declarations are available throughout their containing scope,
/// and function declarations are fully hoisted
fn hoist_declarations(
    ast: &Program,
    context: &mut ScopeAnalysisContext,
) -> AnalysisResult<()> {
    // Hoist all var declarations and function declarations in the global scope
    for statement in &ast.body {
        hoist_statement_declarations(statement, context)?;
    }
    Ok(())
}

/// Hoists declarations within a statement
fn hoist_statement_declarations(
    statement: &Statement,
    context: &mut ScopeAnalysisContext,
) -> AnalysisResult<()> {
    match statement {
        Statement::VariableDeclaration { declarations, kind } => {
            // Only hoist var declarations (let/const are block-scoped)
            if matches!(kind, VariableDeclarationKind::Var) {
                for declarator in declarations {
                    hoist_pattern_declaration(&declarator.id, context)?;
                }
            }
        }
        Statement::FunctionDeclaration { id, .. } => {
            // Function declarations are fully hoisted
            if let Some(function_id) = id {
                declare_symbol(
                    &function_id.name,
                    SymbolType::Function,
                    context.current_scope,
                    context,
                )?;
            }
        }
        Statement::BlockStatement { body } => {
            // Recursively hoist within block statements
            for stmt in body {
                hoist_statement_declarations(stmt, context)?;
            }
        }
        Statement::IfStatement { consequent, alternate, .. } => {
            // Hoist within if statement branches
            hoist_statement_declarations(consequent, context)?;
            if let Some(alt) = alternate {
                hoist_statement_declarations(alt, context)?;
            }
        }
        Statement::WhileStatement { body, .. } => {
            // Hoist within while loop body
            hoist_statement_declarations(body, context)?;
        }
        Statement::ForStatement { body, .. } => {
            // Hoist within for loop body
            hoist_statement_declarations(body, context)?;
        }
        _ => {
            // Other statements don't participate in hoisting
        }
    }
    Ok(())
}

/// Hoists pattern declarations (for destructuring support)
fn hoist_pattern_declaration(
    pattern: &Pattern,
    context: &mut ScopeAnalysisContext,
) -> AnalysisResult<()> {
    match pattern {
        Pattern::Identifier(id) => {
            declare_symbol(
                &id.name,
                SymbolType::Variable { kind: VariableKind::Var },
                context.current_scope,
                context,
            )?;
        }
        // TODO: Handle other pattern types (destructuring)
        _ => {}
    }
    Ok(())
}

/// Analyzes a statement and updates scope information
fn analyze_statement(
    statement: &Statement,
    context: &mut ScopeAnalysisContext,
) -> AnalysisResult<()> {
    match statement {
        Statement::VariableDeclaration { declarations, kind } => {
            analyze_variable_declaration(declarations, kind, context)
        }
        Statement::FunctionDeclaration {
            id,
            params,
            body,
            is_async: _,
            is_generator: _,
        } => analyze_function_declaration(id, params, body, context),
        Statement::ClassDeclaration {
            id,
            super_class,
            body,
        } => analyze_class_declaration(id, super_class, body, context),
        Statement::ExpressionStatement { expression } => {
            analyze_expression(expression, context)
        }
        Statement::BlockStatement { body } => analyze_block_statement(body, context),
        Statement::ReturnStatement { argument } => {
            if let Some(expr) = argument {
                analyze_expression(expr, context)?;
            }
            Ok(())
        }
        Statement::IfStatement {
            test,
            consequent,
            alternate,
        } => {
            analyze_expression(test, context)?;
            analyze_statement(consequent, context)?;
            if let Some(alt) = alternate {
                analyze_statement(alt, context)?;
            }
            Ok(())
        }
        Statement::WhileStatement { test, body } => {
            analyze_expression(test, context)?;
            analyze_statement(body, context)
        }
        Statement::ForStatement {
            init,
            test,
            update,
            body,
        } => analyze_for_statement(init, test, update, body, context),
        Statement::ImportDeclaration { specifiers, source: _ } => {
            analyze_import_declaration(specifiers, context)
        }
        Statement::ExportNamedDeclaration {
            declaration,
            specifiers: _,
            source: _,
        } => {
            if let Some(decl) = declaration {
                analyze_statement(decl, context)?;
                // Mark exported symbols
                mark_last_declaration_as_exported(context);
            }
            Ok(())
        }
    }
}

/// Analyzes variable declarations and adds symbols to current scope
fn analyze_variable_declaration(
    declarations: &[crate::parser::ast_types::VariableDeclarator],
    kind: &VariableDeclarationKind,
    context: &mut ScopeAnalysisContext,
) -> AnalysisResult<()> {
    let var_kind = match kind {
        VariableDeclarationKind::Var => VariableKind::Var,
        VariableDeclarationKind::Let => VariableKind::Let,
        VariableDeclarationKind::Const => VariableKind::Const,
    };

    for declarator in declarations {
        // For var declarations, the symbol was already hoisted
        // For let/const, we need to declare it now
        if !matches!(kind, VariableDeclarationKind::Var) {
            analyze_pattern_binding(&declarator.id, var_kind.clone(), context)?;
        }

        // Analyze initialization expression if present
        if let Some(init) = &declarator.init {
            analyze_expression(init, context)?;
        }
    }

    Ok(())
}

/// Analyzes function declarations and creates new function scope
fn analyze_function_declaration(
    _id: &Option<Identifier>,
    params: &[Pattern],
    body: &crate::parser::ast_types::BlockStatement,
    context: &mut ScopeAnalysisContext,
) -> AnalysisResult<()> {
    // Function name was already hoisted, so no need to re-declare
    
    // Create new function scope
    let function_scope_id = create_scope(ScopeType::Function, Some(context.current_scope), context);

    // Enter function scope
    let previous_scope = context.current_scope;
    context.current_scope = function_scope_id;

    // First, hoist all function declarations and var declarations in this function scope
    for statement in &body.body {
        hoist_statement_declarations(statement, context)?;
    }

    // Bind parameters in function scope
    for param in params {
        analyze_pattern_binding(param, VariableKind::Var, context)?; // Parameters are var-like
    }

    // Analyze function body
    for statement in &body.body {
        analyze_statement(statement, context)?;
    }

    // Restore previous scope
    context.current_scope = previous_scope;

    Ok(())
}

/// Analyzes class declarations and creates new class scope
fn analyze_class_declaration(
    id: &Option<Identifier>,
    super_class: &Option<Box<Expression>>,
    body: &crate::parser::ast_types::ClassBody,
    context: &mut ScopeAnalysisContext,
) -> AnalysisResult<()> {
    // Bind class name in current scope
    if let Some(class_id) = id {
        declare_symbol(
            &class_id.name,
            SymbolType::Class,
            context.current_scope,
            context,
        )?;
    }

    // Analyze super class expression
    if let Some(super_expr) = super_class {
        analyze_expression(super_expr, context)?;
    }

    // Create new class scope
    let class_scope_id = create_scope(ScopeType::Class, Some(context.current_scope), context);

    // Enter class scope
    let previous_scope = context.current_scope;
    context.current_scope = class_scope_id;

    // Analyze class body
    for element in &body.body {
        analyze_class_element(element, context)?;
    }

    // Restore previous scope
    context.current_scope = previous_scope;

    Ok(())
}

/// Analyzes class elements (methods, properties)
fn analyze_class_element(
    element: &ClassElement,
    context: &mut ScopeAnalysisContext,
) -> AnalysisResult<()> {
    match element {
        ClassElement::PropertyDefinition {
            key: _,
            value,
            is_static: _,
            is_private: _,
        } => {
            if let Some(expr) = value {
                analyze_expression(expr, context)?;
            }
            Ok(())
        }
        ClassElement::MethodDefinition {
            key: _,
            value,
            kind: _,
            is_static: _,
            is_private: _,
        } => {
            // Analyze method as function
            analyze_function_expression(value, context)
        }
    }
}

/// Analyzes block statements and creates new block scope for let/const
fn analyze_block_statement(
    body: &[Statement],
    context: &mut ScopeAnalysisContext,
) -> AnalysisResult<()> {
    // Check if we need a new block scope (if there are let/const declarations)
    let needs_block_scope = body.iter().any(|stmt| {
        matches!(
            stmt,
            Statement::VariableDeclaration {
                kind: VariableDeclarationKind::Let | VariableDeclarationKind::Const,
                ..
            }
        )
    });

    if needs_block_scope {
        // Create new block scope
        let block_scope_id = create_scope(ScopeType::Block, Some(context.current_scope), context);

        // Enter block scope
        let previous_scope = context.current_scope;
        context.current_scope = block_scope_id;

        // First, hoist all function declarations and var declarations in this block scope
        for statement in body {
            hoist_statement_declarations(statement, context)?;
        }

        // Analyze statements
        for statement in body {
            analyze_statement(statement, context)?;
        }

        // Restore previous scope
        context.current_scope = previous_scope;
    } else {
        // No new scope needed, just analyze statements
        for statement in body {
            analyze_statement(statement, context)?;
        }
    }

    Ok(())
}

/// Analyzes for statements and handles loop scope
fn analyze_for_statement(
    init: &Option<ForInit>,
    test: &Option<Expression>,
    update: &Option<Expression>,
    body: &Statement,
    context: &mut ScopeAnalysisContext,
) -> AnalysisResult<()> {
    // Check if we need a new scope for loop variables
    let needs_loop_scope = matches!(
        init,
        Some(ForInit::VariableDeclaration {
            kind: VariableDeclarationKind::Let | VariableDeclarationKind::Const,
            ..
        })
    );

    if needs_loop_scope {
        // Create new block scope for loop
        let loop_scope_id = create_scope(ScopeType::Block, Some(context.current_scope), context);

        // Enter loop scope
        let previous_scope = context.current_scope;
        context.current_scope = loop_scope_id;

        // Analyze initialization
        if let Some(for_init) = init {
            analyze_for_init(for_init, context)?;
        }

        // Analyze test condition
        if let Some(test_expr) = test {
            analyze_expression(test_expr, context)?;
        }

        // Analyze update expression
        if let Some(update_expr) = update {
            analyze_expression(update_expr, context)?;
        }

        // Analyze body
        analyze_statement(body, context)?;

        // Restore previous scope
        context.current_scope = previous_scope;
    } else {
        // No new scope needed
        if let Some(for_init) = init {
            analyze_for_init(for_init, context)?;
        }
        if let Some(test_expr) = test {
            analyze_expression(test_expr, context)?;
        }
        if let Some(update_expr) = update {
            analyze_expression(update_expr, context)?;
        }
        analyze_statement(body, context)?;
    }

    Ok(())
}

/// Analyzes for loop initialization
fn analyze_for_init(init: &ForInit, context: &mut ScopeAnalysisContext) -> AnalysisResult<()> {
    match init {
        ForInit::VariableDeclaration { declarations, kind } => {
            analyze_variable_declaration(declarations, kind, context)
        }
        ForInit::Expression(expr) => analyze_expression(expr, context),
    }
}

/// Analyzes import declarations and creates import symbols
fn analyze_import_declaration(
    specifiers: &[ImportSpecifier],
    context: &mut ScopeAnalysisContext,
) -> AnalysisResult<()> {
    for specifier in specifiers {
        match specifier {
            ImportSpecifier::ImportDefaultSpecifier { local } => {
                declare_symbol(&local.name, SymbolType::Import, context.current_scope, context)?;
            }
            ImportSpecifier::ImportNamespaceSpecifier { local } => {
                declare_symbol(&local.name, SymbolType::Import, context.current_scope, context)?;
            }
            ImportSpecifier::ImportSpecifier { imported: _, local } => {
                declare_symbol(&local.name, SymbolType::Import, context.current_scope, context)?;
            }
        }
    }
    Ok(())
}

/// Analyzes expressions and tracks identifier references
fn analyze_expression(expression: &Expression, context: &mut ScopeAnalysisContext) -> AnalysisResult<()> {
    match expression {
        Expression::Identifier(id) => {
            reference_symbol(&id.name, ReferenceType::Read, context);
            Ok(())
        }
        Expression::BinaryExpression { left, right, .. } => {
            analyze_expression(left, context)?;
            analyze_expression(right, context)
        }
        Expression::UnaryExpression { argument, .. } => analyze_expression(argument, context),
        Expression::AssignmentExpression {
            left,
            right,
            operator: _,
        } => {
            // Left side is a write reference
            if let Expression::Identifier(id) = left.as_ref() {
                reference_symbol(&id.name, ReferenceType::Write, context);
            } else {
                analyze_expression(left, context)?;
            }
            analyze_expression(right, context)
        }
        Expression::CallExpression { callee, arguments } => {
            if let Expression::Identifier(id) = callee.as_ref() {
                reference_symbol(&id.name, ReferenceType::Call, context);
            } else {
                analyze_expression(callee, context)?;
            }
            for arg in arguments {
                analyze_expression(arg, context)?;
            }
            Ok(())
        }
        Expression::FunctionExpression(func_expr) => analyze_function_expression(func_expr, context),
        Expression::ArrowFunctionExpression { params, body, .. } => {
            analyze_arrow_function(params, body, context)
        }
        Expression::MemberExpression { object, property, .. } => {
            analyze_expression(object, context)?;
            if let Expression::Identifier(id) = property.as_ref() {
                reference_symbol(&id.name, ReferenceType::PropertyAccess, context);
            } else {
                analyze_expression(property, context)?;
            }
            Ok(())
        }
        Expression::Literal(_) => Ok(()), // Literals don't affect scope
        _ => Ok(()), // Handle other expression types as needed
    }
}

/// Analyzes function expressions
fn analyze_function_expression(
    func_expr: &crate::parser::ast_types::FunctionExpression,
    context: &mut ScopeAnalysisContext,
) -> AnalysisResult<()> {
    // Create new function scope
    let function_scope_id = create_scope(ScopeType::Function, Some(context.current_scope), context);

    // Enter function scope
    let previous_scope = context.current_scope;
    context.current_scope = function_scope_id;

    // First, hoist all function declarations and var declarations in this function scope
    for statement in &func_expr.body.body {
        hoist_statement_declarations(statement, context)?;
    }

    // Bind parameters
    for param in &func_expr.params {
        analyze_pattern_binding(param, VariableKind::Var, context)?;
    }

    // Analyze function body
    for statement in &func_expr.body.body {
        analyze_statement(statement, context)?;
    }

    // Restore previous scope
    context.current_scope = previous_scope;

    Ok(())
}

/// Analyzes arrow function expressions
fn analyze_arrow_function(
    params: &[Pattern],
    body: &crate::parser::ast_types::ArrowFunctionBody,
    context: &mut ScopeAnalysisContext,
) -> AnalysisResult<()> {
    // Create new function scope
    let function_scope_id = create_scope(ScopeType::Function, Some(context.current_scope), context);

    // Enter function scope
    let previous_scope = context.current_scope;
    context.current_scope = function_scope_id;

    // Hoist declarations if it's a block statement
    if let crate::parser::ast_types::ArrowFunctionBody::BlockStatement(block) = body {
        for statement in &block.body {
            hoist_statement_declarations(statement, context)?;
        }
    }

    // Bind parameters
    for param in params {
        analyze_pattern_binding(param, VariableKind::Var, context)?;
    }

    // Analyze arrow function body
    match body {
        crate::parser::ast_types::ArrowFunctionBody::Expression(expr) => {
            analyze_expression(expr, context)?;
        }
        crate::parser::ast_types::ArrowFunctionBody::BlockStatement(block) => {
            for statement in &block.body {
                analyze_statement(statement, context)?;
            }
        }
    }

    // Restore previous scope
    context.current_scope = previous_scope;

    Ok(())
}

/// Analyzes pattern bindings (destructuring, identifiers)
fn analyze_pattern_binding(
    pattern: &Pattern,
    var_kind: VariableKind,
    context: &mut ScopeAnalysisContext,
) -> AnalysisResult<()> {
    match pattern {
        Pattern::Identifier(id) => {
            declare_symbol(
                &id.name,
                SymbolType::Variable { kind: var_kind },
                context.current_scope,
                context,
            )?;
            Ok(())
        }
        // TODO: Handle other pattern types (destructuring, etc.)
        _ => Ok(()),
    }
}

/// Creates a new scope and adds it to the scope tree
fn create_scope(
    scope_type: ScopeType,
    parent_id: Option<ScopeId>,
    context: &mut ScopeAnalysisContext,
) -> ScopeId {
    let scope_id = context.scope_tree.next_id();

    let scope = Scope {
        id: scope_id,
        scope_type,
        parent_id,
        children: Vec::new(),
        bindings: Vec::new(),
        is_safe: true,
    };

    context.scope_tree.scopes.insert(scope_id, scope);

    // Add as child to parent scope
    if let Some(parent) = parent_id
        && let Some(parent_scope) = context.scope_tree.get_scope_mut(parent) {
            parent_scope.children.push(scope_id);
        }

    scope_id
}

/// Declares a new symbol in the specified scope
fn declare_symbol(
    name: &str,
    symbol_type: SymbolType,
    scope_id: ScopeId,
    context: &mut ScopeAnalysisContext,
) -> AnalysisResult<SymbolId> {
    // Check if symbol already exists in this scope (for hoisting)
    if let Some(scope_bindings) = context.symbol_table.scope_bindings.get(&scope_id)
        && let Some(&existing_symbol_id) = scope_bindings.get(name) {
            // Symbol already exists (hoisted), return existing ID
            if context.config.verbose {
                println!("Symbol '{}' already declared in scope {} (hoisted)", name, scope_id);
            }
            return Ok(existing_symbol_id);
        }

    let symbol_id = context.symbol_table.next_id();

    let symbol = Symbol {
        id: symbol_id,
        name: name.to_string(),
        symbol_type,
        scope_id,
        references: Vec::new(),
        is_captured: false,
        is_exported: false,
        is_renamable: true,
    };

    context.symbol_table.symbols.insert(symbol_id, symbol);

    // Add to scope bindings
    context
        .symbol_table
        .scope_bindings
        .entry(scope_id)
        .or_default()
        .insert(name.to_string(), symbol_id);

    // Add to scope's bindings list
    if let Some(scope) = context.scope_tree.get_scope_mut(scope_id) {
        scope.bindings.push(symbol_id);
    }

    if context.config.verbose {
        println!("Declared symbol '{}' in scope {}", name, scope_id);
    }

    Ok(symbol_id)
}

/// Records a reference to a symbol
fn reference_symbol(
    name: &str,
    reference_type: ReferenceType,
    context: &mut ScopeAnalysisContext,
) {
    if context.config.verbose {
        println!("[DEBUG] Recording reference to symbol '{}' of type {:?}", name, reference_type);
    }
    
    // Find the symbol by resolving through scope chain
    if let Some(symbol_id) = resolve_symbol(name, context.current_scope, context) {
        let reference = SymbolReference {
            location: context.current_location.clone(),
            reference_type,
            scope_id: context.current_scope,
        };

        if let Some(symbol) = context.symbol_table.symbols.get_mut(&symbol_id) {
            symbol.references.push(reference);
            
            if context.config.verbose {
                println!("[DEBUG] Added reference to symbol '{}' (id: {}), total references: {}", name, symbol_id, symbol.references.len());
            }

            // Check for closure capture
            if symbol.scope_id != context.current_scope {
                symbol.is_captured = true;
                if context.config.verbose {
                    println!("Symbol '{}' captured by closure", name);
                }
            }
        }
    } else if context.config.verbose {
        println!("Unresolved symbol reference: '{}'", name);
    }
}

/// Resolves a symbol name through the scope chain
fn resolve_symbol(
    name: &str,
    current_scope: ScopeId,
    context: &ScopeAnalysisContext,
) -> Option<SymbolId> {
    let mut scope_id = current_scope;

    loop {
        // Check if symbol exists in current scope
        if let Some(scope_bindings) = context.symbol_table.scope_bindings.get(&scope_id)
            && let Some(&symbol_id) = scope_bindings.get(name) {
                return Some(symbol_id);
            }

        // Move to parent scope
        if let Some(scope) = context.scope_tree.get_scope(scope_id) {
            if let Some(parent_id) = scope.parent_id {
                scope_id = parent_id;
            } else {
                break; // Reached root scope
            }
        } else {
            break;
        }
    }

    None
}

/// Marks the last declared symbol as exported
fn mark_last_declaration_as_exported(context: &mut ScopeAnalysisContext) {
    if let Some(scope) = context.scope_tree.get_scope(context.current_scope)
        && let Some(&last_symbol_id) = scope.bindings.last()
            && let Some(symbol) = context.symbol_table.symbols.get_mut(&last_symbol_id) {
                symbol.is_exported = true;
                symbol.is_renamable = false; // Exported symbols shouldn't be renamed
                if context.config.verbose {
                    println!("Marked symbol '{}' as exported", symbol.name);
                }
            }
}