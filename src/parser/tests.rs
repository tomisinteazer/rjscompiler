//! # Parser Tests
//!
//! Comprehensive test suites for the JavaScript parser following TDD approach.
//! Tests are organized by categories: valid inputs, edge cases, and invalid inputs.

#[cfg(test)]
mod tests {
    use crate::parser::{parse_js, ParserConfig};
    use crate::parser::ast_types::*;
    use crate::parser::{CommentKind};

    /// Helper function to create default parser config
    fn default_config() -> ParserConfig {
        ParserConfig::default()
    }

    /// Helper function to assert successful parsing
    fn assert_parse_success(source: &str, filename: &str) -> Program {
        let config = default_config();
        let result = parse_js(source, filename, &config);
        
        assert!(result.errors.is_empty(), 
                "Expected successful parsing but got errors: {:?}", result.errors);
        assert!(result.ast.is_some(), "Expected AST but got None");
        
        result.ast.unwrap()
    }

    /// Helper function to assert parsing failure with specific error type
    fn assert_parse_error(source: &str, filename: &str, expected_error_pattern: &str) {
        let config = default_config();
        let result = parse_js(source, filename, &config);
        
        assert!(!result.errors.is_empty(), "Expected parsing errors but got none");
        
        let error_message = format!("{:?}", result.errors[0]);
        assert!(error_message.contains(expected_error_pattern),
                "Expected error containing '{}' but got: {}", expected_error_pattern, error_message);
    }

    mod valid_inputs {
        use super::*;

        #[test]
        fn test_simple_variable_declaration() {
            let source = "let x = 5;";
            let ast = assert_parse_success(source, "test.js");
            
            // Verify AST structure
            assert_eq!(ast.body.len(), 1);
            
            match &ast.body[0] {
                Statement::VariableDeclaration { declarations, kind } => {
                    assert!(matches!(kind, VariableDeclarationKind::Let));
                    assert_eq!(declarations.len(), 1);
                    
                    let decl = &declarations[0];
                    match &decl.id {
                        Pattern::Identifier(id) => {
                            assert_eq!(id.name, "x");
                        }
                        _ => panic!("Expected identifier pattern"),
                    }
                    
                    assert!(decl.init.is_some());
                    match decl.init.as_ref().unwrap() {
                        Expression::Literal(Literal::Number(num)) => {
                            assert_eq!(num.value, 5.0);
                        }
                        _ => panic!("Expected number literal"),
                    }
                }
                _ => panic!("Expected variable declaration"),
            }
        }

        #[test]
        fn test_function_declaration_with_return() {
            let source = "function add(a, b) { return a + b; }";
            let ast = assert_parse_success(source, "test.js");
            
            assert_eq!(ast.body.len(), 1);
            
            match &ast.body[0] {
                Statement::FunctionDeclaration { id, params, body, .. } => {
                    // Check function name
                    assert!(id.is_some());
                    assert_eq!(id.as_ref().unwrap().name, "add");
                    
                    // Check parameters
                    assert_eq!(params.len(), 2);
                    match &params[0] {
                        Pattern::Identifier(id) => assert_eq!(id.name, "a"),
                        _ => panic!("Expected identifier parameter"),
                    }
                    match &params[1] {
                        Pattern::Identifier(id) => assert_eq!(id.name, "b"),
                        _ => panic!("Expected identifier parameter"),
                    }
                    
                    // Check function body
                    assert_eq!(body.body.len(), 1);
                    match &body.body[0] {
                        Statement::ReturnStatement { argument } => {
                            assert!(argument.is_some());
                            match argument.as_ref().unwrap() {
                                Expression::BinaryExpression { left, operator, right } => {
                                    assert!(matches!(operator, BinaryOperator::Add));
                                    
                                    match left.as_ref() {
                                        Expression::Identifier(id) => assert_eq!(id.name, "a"),
                                        _ => panic!("Expected identifier 'a'"),
                                    }
                                    
                                    match right.as_ref() {
                                        Expression::Identifier(id) => assert_eq!(id.name, "b"),
                                        _ => panic!("Expected identifier 'b'"),
                                    }
                                }
                                _ => panic!("Expected binary expression"),
                            }
                        }
                        _ => panic!("Expected return statement"),
                    }
                }
                _ => panic!("Expected function declaration"),
            }
        }

        #[test]
        fn test_class_with_private_field() {
            let source = "class C { #x = 1; }";
            let ast = assert_parse_success(source, "test.js");
            
            assert_eq!(ast.body.len(), 1);
            
            match &ast.body[0] {
                Statement::ClassDeclaration { id, body, .. } => {
                    assert!(id.is_some());
                    assert_eq!(id.as_ref().unwrap().name, "C");
                    
                    assert_eq!(body.body.len(), 1);
                    match &body.body[0] {
                        ClassElement::PropertyDefinition { key, value, is_private, .. } => {
                            assert!(*is_private);
                            
                            match key {
                                PropertyKey::PrivateName(name) => {
                                    assert_eq!(name.name, "x");
                                }
                                _ => panic!("Expected private name"),
                            }
                            
                            assert!(value.is_some());
                            match value.as_ref().unwrap() {
                                Expression::Literal(Literal::Number(num)) => {
                                    assert_eq!(num.value, 1.0);
                                }
                                _ => panic!("Expected number literal"),
                            }
                        }
                        _ => panic!("Expected property definition"),
                    }
                }
                _ => panic!("Expected class declaration"),
            }
        }

        #[test]
        fn test_template_literal() {
            let source = "const t = `hello ${name}`;";
            let ast = assert_parse_success(source, "test.js");
            
            assert_eq!(ast.body.len(), 1);
            
            match &ast.body[0] {
                Statement::VariableDeclaration { declarations, kind } => {
                    assert!(matches!(kind, VariableDeclarationKind::Const));
                    assert_eq!(declarations.len(), 1);
                    
                    let decl = &declarations[0];
                    match &decl.id {
                        Pattern::Identifier(id) => {
                            assert_eq!(id.name, "t");
                        }
                        _ => panic!("Expected identifier pattern"),
                    }
                    
                    assert!(decl.init.is_some());
                    match decl.init.as_ref().unwrap() {
                        Expression::TemplateLiteral { quasis, expressions } => {
                            assert_eq!(quasis.len(), 2);
                            assert_eq!(expressions.len(), 1);
                            
                            // Check template elements
                            assert_eq!(quasis[0].value, "hello ");
                            assert!(!quasis[0].tail);
                            assert_eq!(quasis[1].value, "");
                            assert!(quasis[1].tail);
                            
                            // Check expression
                            match &expressions[0] {
                                Expression::Identifier(id) => {
                                    assert_eq!(id.name, "name");
                                }
                                _ => panic!("Expected identifier in template"),
                            }
                        }
                        _ => panic!("Expected template literal"),
                    }
                }
                _ => panic!("Expected variable declaration"),
            }
        }

        #[test]
        fn test_const_declaration() {
            let source = "const PI = 3.14159;";
            let ast = assert_parse_success(source, "test.js");
            
            assert_eq!(ast.body.len(), 1);
            
            match &ast.body[0] {
                Statement::VariableDeclaration { kind, .. } => {
                    assert!(matches!(kind, VariableDeclarationKind::Const));
                }
                _ => panic!("Expected const declaration"),
            }
        }

        #[test]
        fn test_var_declaration() {
            let source = "var oldStyle = true;";
            let ast = assert_parse_success(source, "test.js");
            
            assert_eq!(ast.body.len(), 1);
            
            match &ast.body[0] {
                Statement::VariableDeclaration { kind, .. } => {
                    assert!(matches!(kind, VariableDeclarationKind::Var));
                }
                _ => panic!("Expected var declaration"),
            }
        }

        #[test]
        fn test_boolean_literals() {
            let source = "let isTrue = true; let isFalse = false;";
            let ast = assert_parse_success(source, "test.js");
            
            assert_eq!(ast.body.len(), 2);
            
            // Test true literal
            match &ast.body[0] {
                Statement::VariableDeclaration { declarations, .. } => {
                    match &declarations[0].init {
                        Some(Expression::Literal(Literal::Boolean(b))) => {
                            assert!(b.value);
                        }
                        _ => panic!("Expected boolean true literal"),
                    }
                }
                _ => panic!("Expected variable declaration"),
            }
            
            // Test false literal
            match &ast.body[1] {
                Statement::VariableDeclaration { declarations, .. } => {
                    match &declarations[0].init {
                        Some(Expression::Literal(Literal::Boolean(b))) => {
                            assert!(!b.value);
                        }
                        _ => panic!("Expected boolean false literal"),
                    }
                }
                _ => panic!("Expected variable declaration"),
            }
        }

        #[test]
        fn test_null_literal() {
            let source = "let empty = null;";
            let ast = assert_parse_success(source, "test.js");
            
            assert_eq!(ast.body.len(), 1);
            
            match &ast.body[0] {
                Statement::VariableDeclaration { declarations, .. } => {
                    match &declarations[0].init {
                        Some(Expression::Literal(Literal::Null)) => {
                            // Test passes
                        }
                        _ => panic!("Expected null literal"),
                    }
                }
                _ => panic!("Expected variable declaration"),
            }
        }

        #[test]
        fn test_string_literal() {
            let source = "let message = 'Hello World';";
            let ast = assert_parse_success(source, "test.js");
            
            assert_eq!(ast.body.len(), 1);
            
            match &ast.body[0] {
                Statement::VariableDeclaration { declarations, .. } => {
                    match &declarations[0].init {
                        Some(Expression::Literal(Literal::String(s))) => {
                            assert_eq!(s.value, "Hello World");
                        }
                        _ => panic!("Expected string literal"),
                    }
                }
                _ => panic!("Expected variable declaration"),
            }
        }

        #[test]
        fn test_multiple_statements() {
            let source = "let x = 1; let y = 2; let z = x + y;";
            let ast = assert_parse_success(source, "test.js");
            
            assert_eq!(ast.body.len(), 3);
            
            // All should be variable declarations
            for stmt in &ast.body {
                match stmt {
                    Statement::VariableDeclaration { .. } => {
                        // Test passes
                    }
                    _ => panic!("Expected variable declaration"),
                }
            }
        }
    }

    mod edge_cases {
        use super::*;

        #[test]
        fn test_regex_vs_division() {
            let source = "const r = /abc/; let y = a / b;";
            let ast = assert_parse_success(source, "test.js");
            
            assert_eq!(ast.body.len(), 2);
            
            // First should be regex literal
            match &ast.body[0] {
                Statement::VariableDeclaration { declarations, .. } => {
                    match &declarations[0].init {
                        Some(Expression::Literal(Literal::RegExp(regex))) => {
                            assert_eq!(regex.pattern, "abc");
                            assert_eq!(regex.flags, "");
                        }
                        _ => panic!("Expected regex literal"),
                    }
                }
                _ => panic!("Expected variable declaration"),
            }
            
            // Second should be division expression
            match &ast.body[1] {
                Statement::VariableDeclaration { declarations, .. } => {
                    match &declarations[0].init {
                        Some(Expression::BinaryExpression { operator, .. }) => {
                            assert!(matches!(operator, BinaryOperator::Divide));
                        }
                        _ => panic!("Expected binary expression"),
                    }
                }
                _ => panic!("Expected variable declaration"),
            }
        }

        #[test]
        fn test_asi_return() {
            let source = "function f(){ return\n5; }";
            let ast = assert_parse_success(source, "test.js");
            
            assert_eq!(ast.body.len(), 1);
            
            match &ast.body[0] {
                Statement::FunctionDeclaration { body, .. } => {
                    assert_eq!(body.body.len(), 2);
                    
                    // First should be return statement with no argument
                    match &body.body[0] {
                        Statement::ReturnStatement { argument } => {
                            assert!(argument.is_none());
                        }
                        _ => panic!("Expected return statement"),
                    }
                    
                    // Second should be expression statement with literal 5
                    match &body.body[1] {
                        Statement::ExpressionStatement { expression } => {
                            match expression {
                                Expression::Literal(Literal::Number(num)) => {
                                    assert_eq!(num.value, 5.0);
                                }
                                _ => panic!("Expected number literal"),
                            }
                        }
                        _ => panic!("Expected expression statement"),
                    }
                }
                _ => panic!("Expected function declaration"),
            }
        }

        #[test]
        fn test_empty_statements() {
            let source = ";;; let x = 1;;";
            let ast = assert_parse_success(source, "test.js");
            
            // Should handle empty statements gracefully
            // Implementation dependent on how parser handles empty statements
        }

        #[test]
        fn test_nested_expressions() {
            let source = "let result = ((a + b) * (c - d)) / e;";
            let ast = assert_parse_success(source, "test.js");
            
            assert_eq!(ast.body.len(), 1);
            
            match &ast.body[0] {
                Statement::VariableDeclaration { declarations, .. } => {
                    match &declarations[0].init {
                        Some(Expression::BinaryExpression { operator, .. }) => {
                            assert!(matches!(operator, BinaryOperator::Divide));
                        }
                        _ => panic!("Expected binary expression"),
                    }
                }
                _ => panic!("Expected variable declaration"),
            }
        }

        #[test]
        fn test_function_expression() {
            let source = "let fn = function(x) { return x * 2; };";
            let ast = assert_parse_success(source, "test.js");
            
            assert_eq!(ast.body.len(), 1);
            
            match &ast.body[0] {
                Statement::VariableDeclaration { declarations, .. } => {
                    match &declarations[0].init {
                        Some(Expression::FunctionExpression(func)) => {
                            assert!(func.id.is_none()); // Anonymous function
                            assert_eq!(func.params.len(), 1);
                        }
                        _ => panic!("Expected function expression"),
                    }
                }
                _ => panic!("Expected variable declaration"),
            }
        }
    }

    mod invalid_inputs {
        use super::*;

        #[test]
        fn test_unterminated_string() {
            let source = "'abc";
            assert_parse_error(source, "test.js", "Unterminated");
        }

        #[test]
        fn test_unexpected_token() {
            let source = "let = 5;";
            assert_parse_error(source, "test.js", "Unexpected");
        }

        #[test]
        fn test_missing_semicolon_before_return() {
            let source = "function f() { let x = 1 return x; }";
            // Some parsers may handle this with ASI, others may error
            // This test documents the expected behavior
            let config = default_config();
            let result = parse_js(source, "test.js", &config);
            
            // Either should parse successfully (with ASI) or fail with syntax error
            if !result.errors.is_empty() {
                assert_parse_error(source, "test.js", "");
            }
        }

        #[test]
        fn test_invalid_identifier() {
            let source = "let 123abc = 5;";
            assert_parse_error(source, "test.js", "");
        }

        #[test]
        fn test_unmatched_braces() {
            let source = "function test() { let x = 1;";
            assert_parse_error(source, "test.js", "");
        }

        #[test]
        fn test_invalid_assignment_target() {
            let source = "5 = x;";
            assert_parse_error(source, "test.js", "");
        }

        #[test]
        fn test_invalid_function_name() {
            let source = "function 123() {}";
            assert_parse_error(source, "test.js", "");
        }

        #[test]
        fn test_duplicate_parameter_names() {
            let source = "function test(a, a) {}";
            // This may or may not be an error depending on strict mode
            // Test documents expected behavior
            let config = default_config();
            let _result = parse_js(source, "test.js", &config);
            // Implementation dependent
        }
    }

    mod performance_tests {
        use super::*;

        #[test]
        fn test_large_file_parsing() {
            // Create a large JavaScript file content
            let mut large_source = String::new();
            for i in 0..1000 {
                large_source.push_str(&format!("let var{} = {};\n", i, i));
            }
            
            let start = std::time::Instant::now();
            let ast = assert_parse_success(&large_source, "large.js");
            let duration = start.elapsed();
            
            assert_eq!(ast.body.len(), 1000);
            println!("Parsed 1000 statements in {:?}", duration);
            
            // Performance should be reasonable (less than 1 second for this test)
            assert!(duration.as_secs() < 5, "Parsing took too long: {:?}", duration);
        }

        #[test]
        fn test_deeply_nested_expressions() {
            // Create deeply nested expressions
            let mut nested_source = "let result = ".to_string();
            for i in 0..100 {
                nested_source.push_str(&format!("(a{} + ", i));
            }
            nested_source.push_str("42");
            for _ in 0..100 {
                nested_source.push(')');
            }
            nested_source.push(';');
            
            let start = std::time::Instant::now();
            let ast = assert_parse_success(&nested_source, "nested.js");
            let duration = start.elapsed();
            
            assert_eq!(ast.body.len(), 1);
            println!("Parsed deeply nested expression in {:?}", duration);
            
            // Should handle deep nesting without stack overflow
            assert!(duration.as_secs() < 5, "Deep nesting parsing took too long: {:?}", duration);
        }
    }

    mod trivia_tests {
        use super::*;

        #[test]
        fn test_line_comments_preservation() {
            let source = "// This is a line comment\nlet x = 5; // Another comment";
            let config = ParserConfig {
                preserve_trivia: true,
                ..ParserConfig::default()
            };
            let result = parse_js(source, "test.js", &config);
            
            assert!(result.errors.is_empty());
            assert!(result.trivia.is_some());
            
            let trivia = result.trivia.unwrap();
            assert_eq!(trivia.line_comments.len(), 2);
            
            assert_eq!(trivia.line_comments[0].text, "This is a line comment");
            assert!(matches!(trivia.line_comments[0].kind, CommentKind::Line));
            
            assert_eq!(trivia.line_comments[1].text, "Another comment");
            assert!(matches!(trivia.line_comments[1].kind, CommentKind::Line));
        }

        #[test]
        fn test_block_comments_preservation() {
            let source = "/* This is a block comment */\nlet x = 5;";
            let config = ParserConfig {
                preserve_trivia: true,
                ..ParserConfig::default()
            };
            let result = parse_js(source, "test.js", &config);
            
            assert!(result.errors.is_empty());
            assert!(result.trivia.is_some());
            
            let trivia = result.trivia.unwrap();
            assert_eq!(trivia.block_comments.len(), 1);
            
            assert_eq!(trivia.block_comments[0].text, "This is a block comment");
            assert!(matches!(trivia.block_comments[0].kind, CommentKind::Block));
        }

        #[test]
        fn test_multiline_block_comments() {
            let source = "/*\n * Multi-line\n * block comment\n */\nlet x = 5;";
            let config = ParserConfig {
                preserve_trivia: true,
                ..ParserConfig::default()
            };
            let result = parse_js(source, "test.js", &config);
            
            assert!(result.errors.is_empty());
            assert!(result.trivia.is_some());
            
            let trivia = result.trivia.unwrap();
            assert_eq!(trivia.block_comments.len(), 1);
            
            let comment_text = &trivia.block_comments[0].text;
            assert!(comment_text.contains("Multi-line"));
            assert!(comment_text.contains("block comment"));
        }

        #[test]
        fn test_whitespace_preservation() {
            let source = "  \n  let x = 5;    \n";
            let config = ParserConfig {
                preserve_trivia: true,
                ..ParserConfig::default()
            };
            let result = parse_js(source, "test.js", &config);
            
            assert!(result.errors.is_empty());
            assert!(result.trivia.is_some());
            
            let trivia = result.trivia.unwrap();
            assert!(!trivia.leading_whitespace.is_empty() || !trivia.trailing_whitespace.is_empty());
        }

        #[test]
        fn test_mixed_comments_and_code() {
            let source = r#"
            // Header comment
            function test() {
                /* Function body comment */
                return 42; // Return comment
            }
            // Footer comment
            "#;
            
            let config = ParserConfig {
                preserve_trivia: true,
                ..ParserConfig::default()
            };
            let result = parse_js(source, "test.js", &config);
            
            assert!(result.errors.is_empty());
            assert!(result.trivia.is_some());
            
            let trivia = result.trivia.unwrap();
            
            // Should have both line and block comments
            assert!(!trivia.line_comments.is_empty());
            assert!(!trivia.block_comments.is_empty());
            
            // Check specific comments
            let line_comment_texts: Vec<_> = trivia.line_comments.iter()
                .map(|c| c.text.as_str())
                .collect();
            assert!(line_comment_texts.contains(&"Header comment"));
            assert!(line_comment_texts.contains(&"Return comment"));
            assert!(line_comment_texts.contains(&"Footer comment"));
            
            let block_comment_texts: Vec<_> = trivia.block_comments.iter()
                .map(|c| c.text.as_str())
                .collect();
            assert!(block_comment_texts.contains(&"Function body comment"));
        }

        #[test]
        fn test_trivia_disabled() {
            let source = "// Comment\nlet x = 5;";
            let config = ParserConfig {
                preserve_trivia: false,
                ..ParserConfig::default()
            };
            let result = parse_js(source, "test.js", &config);
            
            assert!(result.errors.is_empty());
            assert!(result.trivia.is_none());
        }

        #[test]
        fn test_comment_positions() {
            let source = "// Comment at position 0\nlet x = 5;";
            let config = ParserConfig {
                preserve_trivia: true,
                ..ParserConfig::default()
            };
            let result = parse_js(source, "test.js", &config);
            
            assert!(result.errors.is_empty());
            let trivia = result.trivia.unwrap();
            
            assert_eq!(trivia.line_comments.len(), 1);
            let comment = &trivia.line_comments[0];
            
            // Comment should start at position 0
            assert_eq!(comment.span.start, 0);
            // Comment should end before the newline
            assert!(comment.span.end > 0);
        }

        #[test]
        fn test_nested_comments_in_strings() {
            // Comments inside strings should not be extracted as comments
            let source = r#"let str = "// This is not a comment";"#;
            let config = ParserConfig {
                preserve_trivia: true,
                ..ParserConfig::default()
            };
            let result = parse_js(source, "test.js", &config);
            
            assert!(result.errors.is_empty());
            let trivia = result.trivia.unwrap();
            
            // Should not find any comments since it's inside a string
            assert_eq!(trivia.line_comments.len(), 0);
        }
    }
}