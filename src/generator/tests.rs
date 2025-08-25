//! # Generator Component Tests
//!
//! Comprehensive test suite for the JavaScript generator component following TDD approach.
//! Tests cover all AST node types, precedence handling, ASI hazards, string processing,
//! and source map generation.

use crate::generator::{Generator, GeneratorConfig, OutputFormat, SemicolonStrategy, QuoteStrategy};
use crate::parser::ast_types::*;

/// Test helper for creating AST nodes
pub struct AstTestBuilder;

impl AstTestBuilder {
    /// Create a simple identifier
    pub fn identifier(name: &str) -> Identifier {
        Identifier { name: name.to_string() }
    }

    /// Create a number literal
    pub fn number(value: f64) -> Expression {
        Expression::Literal(Literal::Number(NumberLiteral { value }))
    }

    /// Create a string literal
    pub fn string(value: &str) -> Expression {
        Expression::Literal(Literal::String(StringLiteral { value: value.to_string() }))
    }

    /// Create a boolean literal
    pub fn boolean(value: bool) -> Expression {
        Expression::Literal(Literal::Boolean(BooleanLiteral { value }))
    }

    /// Create a null literal
    pub fn null() -> Expression {
        Expression::Literal(Literal::Null)
    }

    /// Create a variable declaration
    pub fn var_declaration(name: &str, init: Option<Expression>, kind: VariableDeclarationKind) -> Statement {
        Statement::VariableDeclaration {
            declarations: vec![VariableDeclarator {
                id: Pattern::Identifier(Self::identifier(name)),
                init,
            }],
            kind,
        }
    }

    /// Create a binary expression
    pub fn binary_expr(left: Expression, op: BinaryOperator, right: Expression) -> Expression {
        Expression::BinaryExpression {
            left: Box::new(left),
            operator: op,
            right: Box::new(right),
        }
    }

    /// Create an identifier expression
    pub fn id_expr(name: &str) -> Expression {
        Expression::Identifier(Self::identifier(name))
    }

    /// Create a simple program
    pub fn program(statements: Vec<Statement>) -> Program {
        Program {
            body: statements,
            source_type: ProgramSourceType::Script,
        }
    }
}

/// Performance optimization tests
/// Tests cover string builders, memory management, caching, and efficiency improvements.
#[cfg(test)]
mod performance_tests {
    use super::*;
    use crate::generator::printer::Printer;

    /// Test performance metrics collection
    #[test]
    fn test_performance_metrics() {
        let config = GeneratorConfig::default();
        let mut printer = Printer::new(&config);
        
        let program = AstTestBuilder::program(vec![
            AstTestBuilder::var_declaration("x", Some(AstTestBuilder::number(5.0)), VariableDeclarationKind::Let)
        ]);
        
        let _result = printer.print_program(&program).unwrap();
        let metrics = printer.get_performance_metrics();
        
        // Verify metrics are being tracked
        assert!(metrics.chars_written > 0);
        assert!(metrics.output_capacity > 0);
        assert!(metrics.output_length > 0);
        assert_eq!(metrics.output_length, metrics.chars_written);
    }

    /// Test memory pre-allocation optimization
    #[test]
    fn test_memory_preallocation() {
        let config = GeneratorConfig {
            format: crate::generator::OutputFormat::Pretty,
            ..GeneratorConfig::default()
        };
        let printer = Printer::new(&config);
        let metrics = printer.get_performance_metrics();
        
        // Pretty format should pre-allocate more capacity
        assert!(metrics.output_capacity >= 8192);
    }

    /// Test compact format capacity optimization
    #[test]
    fn test_compact_format_optimization() {
        let config = GeneratorConfig {
            format: crate::generator::OutputFormat::Compact,
            ..GeneratorConfig::default()
        };
        let printer = Printer::new(&config);
        let metrics = printer.get_performance_metrics();
        
        // Compact format should use smaller initial capacity
        assert!(metrics.output_capacity >= 2048);
        assert!(metrics.output_capacity < 8192); // Should be less than pretty format
    }

    /// Test printer reset functionality
    #[test]
    fn test_printer_reset() {
        let config = GeneratorConfig::default();
        let mut printer = Printer::new(&config);
        
        // Generate some content
        let program = AstTestBuilder::program(vec![
            AstTestBuilder::var_declaration("x", Some(AstTestBuilder::number(5.0)), VariableDeclarationKind::Let)
        ]);
        let _result = printer.print_program(&program).unwrap();
        
        // Verify content was generated
        let metrics_before = printer.get_performance_metrics();
        assert!(metrics_before.chars_written > 0);
        assert!(metrics_before.output_length > 0);
        
        // Reset the printer
        printer.reset();
        
        // Verify state was reset
        let metrics_after = printer.get_performance_metrics();
        assert_eq!(metrics_after.chars_written, 0);
        assert_eq!(metrics_after.output_length, 0);
        // Capacity should be preserved for reuse
        assert!(metrics_after.output_capacity > 0);
    }

    /// Test large program performance
    #[test]
    fn test_large_program_performance() {
        let config = GeneratorConfig::default();
        let mut printer = Printer::new(&config);
        
        // Create a large program with many statements
        let statements: Vec<Statement> = (0..1000)
            .map(|i| {
                AstTestBuilder::var_declaration(
                    &format!("var{}", i),
                    Some(AstTestBuilder::number(i as f64)),
                    VariableDeclarationKind::Let
                )
            })
            .collect();
        
        let program = AstTestBuilder::program(statements);
        
        let start_time = std::time::Instant::now();
        let result = printer.print_program(&program).unwrap();
        let generation_time = start_time.elapsed();
        
        // Should handle large programs efficiently
        assert!(generation_time.as_millis() < 100); // Should be fast
        assert!(result.len() > 10000); // Should generate substantial output
        
        let metrics = printer.get_performance_metrics();
        assert_eq!(metrics.chars_written, result.len());
    }

    /// Test memory usage with complex nested structures
    #[test]
    fn test_nested_structure_performance() {
        let config = GeneratorConfig::default();
        let mut printer = Printer::new(&config);
        
        // Create deeply nested block statements
        let mut nested_stmt = Statement::ExpressionStatement {
            expression: AstTestBuilder::number(1.0)
        };
        
        for _ in 0..10 {
            nested_stmt = Statement::BlockStatement {
                body: vec![nested_stmt]
            };
        }
        
        let program = AstTestBuilder::program(vec![nested_stmt]);
        let result = printer.print_program(&program).unwrap();
        
        // Should handle nested structures without excessive memory usage
        let metrics = printer.get_performance_metrics();
        assert!(metrics.output_capacity < 100000); // Shouldn't over-allocate
        assert!(result.contains("{"));
        assert!(result.contains("}"));
    }

    /// Test template literal performance optimization
    #[test]
    fn test_template_literal_performance() {
        let config = GeneratorConfig::default();
        let mut printer = Printer::new(&config);
        
        // Create template literal with multiple expressions
        let template_expr = Expression::TemplateLiteral {
            quasis: vec![
                TemplateElement { value: "Hello ".to_string(), tail: false },
                TemplateElement { value: ", you are ".to_string(), tail: false },
                TemplateElement { value: " years old!".to_string(), tail: true },
            ],
            expressions: vec![
                AstTestBuilder::id_expr("name"),
                AstTestBuilder::id_expr("age"),
            ],
        };
        
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement { expression: template_expr }
        ]);
        
        let result = printer.print_program(&program).unwrap();
        
        // Verify template literal was processed correctly
        assert!(result.contains("`Hello ${name}, you are ${age} years old!`;"));
        
        let metrics = printer.get_performance_metrics();
        assert!(metrics.chars_written > 0);
    }

    /// Test string processing performance
    #[test]
    fn test_string_processing_performance() {
        let config = GeneratorConfig::default();
        let mut printer = Printer::new(&config);
        
        // Create program with many string literals
        let strings: Vec<Statement> = (0..100)
            .map(|i| {
                Statement::ExpressionStatement {
                    expression: AstTestBuilder::string(&format!("String number {} with special chars \\n\\t", i))
                }
            })
            .collect();
        
        let program = AstTestBuilder::program(strings);
        
        let start_time = std::time::Instant::now();
        let result = printer.print_program(&program).unwrap();
        let processing_time = start_time.elapsed();
        
        // Should process many strings efficiently
        assert!(processing_time.as_millis() < 50);
        assert!(result.contains("String number 0"));
        assert!(result.contains("String number 99"));
        
        let metrics = printer.get_performance_metrics();
        assert!(metrics.chars_written > 1000);
    }

    /// Test cache utilization
    #[test]
    fn test_cache_utilization() {
        let config = GeneratorConfig {
            format: crate::generator::OutputFormat::Pretty,
            ..GeneratorConfig::default()
        };
        let mut printer = Printer::new(&config);
        
        // Create nested block structure to test indent caching
        let mut nested_body = vec![Statement::ExpressionStatement {
            expression: AstTestBuilder::number(1.0)
        }];
        
        for _ in 0..5 {
            nested_body = vec![Statement::BlockStatement { body: nested_body }];
        }
        
        let program = AstTestBuilder::program(nested_body);
        let _result = printer.print_program(&program).unwrap();
        
        let metrics = printer.get_performance_metrics();
        // Cache utilization should be reasonable for nested structures
        assert!(metrics.cache_utilization >= 0.0);
        assert!(metrics.cache_utilization <= 1.0);
    }

    /// Test performance with different output formats
    #[test]
    fn test_format_performance_comparison() {
        let test_program = AstTestBuilder::program(vec![
            AstTestBuilder::var_declaration("x", Some(AstTestBuilder::number(1.0)), VariableDeclarationKind::Let),
            AstTestBuilder::var_declaration("y", Some(AstTestBuilder::number(2.0)), VariableDeclarationKind::Const),
        ]);
        
        // Test compact format
        let compact_config = GeneratorConfig {
            format: crate::generator::OutputFormat::Compact,
            ..GeneratorConfig::default()
        };
        let mut compact_printer = Printer::new(&compact_config);
        let compact_result = compact_printer.print_program(&test_program).unwrap();
        let compact_metrics = compact_printer.get_performance_metrics();
        
        // Test pretty format
        let pretty_config = GeneratorConfig {
            format: crate::generator::OutputFormat::Pretty,
            ..GeneratorConfig::default()
        };
        let mut pretty_printer = Printer::new(&pretty_config);
        let pretty_result = pretty_printer.print_program(&test_program).unwrap();
        let pretty_metrics = pretty_printer.get_performance_metrics();
        
        // Compact should generate less output
        assert!(compact_result.len() < pretty_result.len());
        assert!(compact_metrics.chars_written < pretty_metrics.chars_written);
        
        // But pretty format should have higher initial capacity
        assert!(pretty_metrics.output_capacity > compact_metrics.output_capacity);
    }

    /// Test memory efficiency with reused printer
    #[test]
    fn test_printer_reuse_efficiency() {
        let config = GeneratorConfig::default();
        let mut printer = Printer::new(&config);
        
        let test_program = AstTestBuilder::program(vec![
            AstTestBuilder::var_declaration("x", Some(AstTestBuilder::number(1.0)), VariableDeclarationKind::Let)
        ]);
        
        // First generation
        let _result1 = printer.print_program(&test_program).unwrap();
        let capacity_after_first = printer.get_performance_metrics().output_capacity;
        
        // Reset and generate again
        printer.reset();
        let _result2 = printer.print_program(&test_program).unwrap();
        let capacity_after_second = printer.get_performance_metrics().output_capacity;
        
        // Capacity should be preserved for efficiency
        assert_eq!(capacity_after_first, capacity_after_second);
    }
}

/// Generator test suite
#[cfg(test)]
mod tests {
    use super::*;

    /// Test generator configuration creation
    #[test]
    fn test_generator_config_creation() {
        let config = GeneratorConfig::default();
        assert!(matches!(config.format, OutputFormat::Compact));
        assert!(matches!(config.semicolon, SemicolonStrategy::Auto));
        assert!(matches!(config.quote, QuoteStrategy::Auto));
    }

    /// Test generator instantiation
    #[test]
    fn test_generator_creation() {
        let config = GeneratorConfig::default();
        let generator = Generator::new(config);
        // Generator should be created successfully
        assert!(std::ptr::addr_of!(generator) as usize != 0);
    }

    /// Test empty program generation
    #[test]
    fn test_empty_program_generation() {
        let config = GeneratorConfig::default();
        let generator = Generator::new(config);
        let program = AstTestBuilder::program(vec![]);

        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "");
        assert_eq!(result.diagnostics.generated_size, 0);
    }

    /// Golden tests for variable declarations
    #[test]
    fn test_var_declaration_let() {
        let config = GeneratorConfig::default();
        let generator = Generator::new(config);
        
        let program = AstTestBuilder::program(vec![
            AstTestBuilder::var_declaration("x", Some(AstTestBuilder::number(5.0)), VariableDeclarationKind::Let)
        ]);

        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "let x=5;");
    }

    #[test]
    fn test_var_declaration_const() {
        let config = GeneratorConfig::default();
        let generator = Generator::new(config);
        
        let program = AstTestBuilder::program(vec![
            AstTestBuilder::var_declaration("PI", Some(AstTestBuilder::number(3.14)), VariableDeclarationKind::Const)
        ]);

        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "const PI=3.14;");
    }

    #[test]
    fn test_var_declaration_var() {
        let config = GeneratorConfig::default();
        let generator = Generator::new(config);
        
        let program = AstTestBuilder::program(vec![
            AstTestBuilder::var_declaration("count", Some(AstTestBuilder::number(0.0)), VariableDeclarationKind::Var)
        ]);

        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "var count=0;");
    }

    #[test]
    fn test_var_declaration_no_init() {
        let config = GeneratorConfig::default();
        let generator = Generator::new(config);
        
        let program = AstTestBuilder::program(vec![
            AstTestBuilder::var_declaration("x", None, VariableDeclarationKind::Let)
        ]);

        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "let x;");
    }

    /// Golden tests for literals
    #[test]
    fn test_number_literals() {
        let config = GeneratorConfig::default();
        let generator = Generator::new(config);
        
        // Integer
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement { expression: AstTestBuilder::number(42.0) }
        ]);
        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "42;");

        // Float
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement { expression: AstTestBuilder::number(3.14) }
        ]);
        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "3.14;");

        // Zero
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement { expression: AstTestBuilder::number(0.0) }
        ]);
        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "0;");
    }

    #[test]
    fn test_string_literals() {
        let config = GeneratorConfig::default();
        let generator = Generator::new(config);
        
        // Simple string
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement { expression: AstTestBuilder::string("hello") }
        ]);
        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "'hello';");

        // String with single quotes (should use double quotes)
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement { expression: AstTestBuilder::string("it's") }
        ]);
        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "\"it's\";");

        // Empty string
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement { expression: AstTestBuilder::string("") }
        ]);
        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "'';");
    }

    #[test]
    fn test_boolean_literals() {
        let config = GeneratorConfig::default();
        let generator = Generator::new(config);
        
        // True
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement { expression: AstTestBuilder::boolean(true) }
        ]);
        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "true;");

        // False
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement { expression: AstTestBuilder::boolean(false) }
        ]);
        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "false;");
    }

    #[test]
    fn test_null_literal() {
        let config = GeneratorConfig::default();
        let generator = Generator::new(config);
        
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement { expression: AstTestBuilder::null() }
        ]);
        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "null;");
    }

    /// Golden tests for binary expressions
    #[test]
    fn test_binary_expressions_arithmetic() {
        let config = GeneratorConfig::default();
        let generator = Generator::new(config);
        
        // Addition
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement { 
                expression: AstTestBuilder::binary_expr(
                    AstTestBuilder::number(1.0),
                    BinaryOperator::Add,
                    AstTestBuilder::number(2.0)
                )
            }
        ]);
        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "1+2;");

        // Subtraction
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement { 
                expression: AstTestBuilder::binary_expr(
                    AstTestBuilder::number(5.0),
                    BinaryOperator::Subtract,
                    AstTestBuilder::number(3.0)
                )
            }
        ]);
        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "5-3;");

        // Multiplication
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement { 
                expression: AstTestBuilder::binary_expr(
                    AstTestBuilder::number(4.0),
                    BinaryOperator::Multiply,
                    AstTestBuilder::number(6.0)
                )
            }
        ]);
        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "4*6;");

        // Division
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement { 
                expression: AstTestBuilder::binary_expr(
                    AstTestBuilder::number(10.0),
                    BinaryOperator::Divide,
                    AstTestBuilder::number(2.0)
                )
            }
        ]);
        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "10/2;");

        // Remainder
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement { 
                expression: AstTestBuilder::binary_expr(
                    AstTestBuilder::number(7.0),
                    BinaryOperator::Remainder,
                    AstTestBuilder::number(3.0)
                )
            }
        ]);
        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "7%3;");
    }

    #[test]
    fn test_binary_expressions_multiplication_division() {
        let config = GeneratorConfig::default();
        let generator = Generator::new(config);
        
        // Multiplication
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement { 
                expression: AstTestBuilder::binary_expr(
                    AstTestBuilder::number(4.0),
                    BinaryOperator::Multiply,
                    AstTestBuilder::number(6.0)
                )
            }
        ]);
        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "4*6;");

        // Division
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement { 
                expression: AstTestBuilder::binary_expr(
                    AstTestBuilder::number(8.0),
                    BinaryOperator::Divide,
                    AstTestBuilder::number(2.0)
                )
            }
        ]);
        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "8/2;");
    }

    #[test]
    fn test_binary_expressions_comparison() {
        let config = GeneratorConfig::default();
        let generator = Generator::new(config);
        
        // Equality
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement { 
                expression: AstTestBuilder::binary_expr(
                    AstTestBuilder::id_expr("a"),
                    BinaryOperator::Equal,
                    AstTestBuilder::id_expr("b")
                )
            }
        ]);
        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "a==b;");

        // Strict equality
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement { 
                expression: AstTestBuilder::binary_expr(
                    AstTestBuilder::id_expr("x"),
                    BinaryOperator::StrictEqual,
                    AstTestBuilder::number(5.0)
                )
            }
        ]);
        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "x===5;");

        // Less than
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement { 
                expression: AstTestBuilder::binary_expr(
                    AstTestBuilder::id_expr("y"),
                    BinaryOperator::LessThan,
                    AstTestBuilder::number(10.0)
                )
            }
        ]);
        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "y<10;");
    }

    #[test]
    fn test_binary_expressions_logical() {
        let config = GeneratorConfig::default();
        let generator = Generator::new(config);
        
        // Logical AND
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement { 
                expression: AstTestBuilder::binary_expr(
                    AstTestBuilder::id_expr("a"),
                    BinaryOperator::LogicalAnd,
                    AstTestBuilder::id_expr("b")
                )
            }
        ]);
        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "a&&b;");

        // Logical OR
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement { 
                expression: AstTestBuilder::binary_expr(
                    AstTestBuilder::id_expr("x"),
                    BinaryOperator::LogicalOr,
                    AstTestBuilder::id_expr("y")
                )
            }
        ]);
        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "x||y;");
    }

    /// Test operator precedence with parentheses
    #[test]
    fn test_operator_precedence() {
        let config = GeneratorConfig::default();
        let generator = Generator::new(config);
        
        // Addition and multiplication: a + b * c should not need parens
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement { 
                expression: AstTestBuilder::binary_expr(
                    AstTestBuilder::id_expr("a"),
                    BinaryOperator::Add,
                    AstTestBuilder::binary_expr(
                        AstTestBuilder::id_expr("b"),
                        BinaryOperator::Multiply,
                        AstTestBuilder::id_expr("c")
                    )
                )
            }
        ]);
        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "a+b*c;");

        // Multiplication and addition: (a + b) * c should need parens
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement { 
                expression: AstTestBuilder::binary_expr(
                    AstTestBuilder::binary_expr(
                        AstTestBuilder::id_expr("a"),
                        BinaryOperator::Add,
                        AstTestBuilder::id_expr("b")
                    ),
                    BinaryOperator::Multiply,
                    AstTestBuilder::id_expr("c")
                )
            }
        ]);
        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "(a+b)*c;");
    }

    /// Test multiple statements
    #[test]
    fn test_multiple_statements() {
        let config = GeneratorConfig::default();
        let generator = Generator::new(config);
        
        let program = AstTestBuilder::program(vec![
            AstTestBuilder::var_declaration("x", Some(AstTestBuilder::number(1.0)), VariableDeclarationKind::Let),
            AstTestBuilder::var_declaration("y", Some(AstTestBuilder::number(2.0)), VariableDeclarationKind::Let),
            Statement::ExpressionStatement { 
                expression: AstTestBuilder::binary_expr(
                    AstTestBuilder::id_expr("x"),
                    BinaryOperator::Add,
                    AstTestBuilder::id_expr("y")
                )
            }
        ]);

        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "let x=1;let y=2;x+y;");
    }

    /// Test different output formats
    #[test]
    fn test_output_format_compact() {
        let mut config = GeneratorConfig::default();
        config.format = OutputFormat::Compact;
        let generator = Generator::new(config);
        
        let program = AstTestBuilder::program(vec![
            AstTestBuilder::var_declaration("x", Some(AstTestBuilder::number(5.0)), VariableDeclarationKind::Let)
        ]);

        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "let x=5;");
    }

    #[test]
    fn test_output_format_readable() {
        let mut config = GeneratorConfig::default();
        config.format = OutputFormat::Readable;
        let generator = Generator::new(config);
        
        let program = AstTestBuilder::program(vec![
            AstTestBuilder::var_declaration("x", Some(AstTestBuilder::number(5.0)), VariableDeclarationKind::Let)
        ]);

        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "let x = 5;\n");
    }

    /// Test quote strategies
    #[test]
    fn test_quote_strategy_single() {
        let mut config = GeneratorConfig::default();
        config.quote = QuoteStrategy::Single;
        let generator = Generator::new(config);
        
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement { expression: AstTestBuilder::string("hello") }
        ]);

        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "'hello';");
    }

    #[test]
    fn test_quote_strategy_double() {
        let mut config = GeneratorConfig::default();
        config.quote = QuoteStrategy::Double;
        let generator = Generator::new(config);
        
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement { expression: AstTestBuilder::string("hello") }
        ]);

        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "\"hello\";");
    }

    /// Test semicolon strategies
    #[test]
    fn test_semicolon_strategy_always() {
        let mut config = GeneratorConfig::default();
        config.semicolon = SemicolonStrategy::Always;
        let generator = Generator::new(config);
        
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement { expression: AstTestBuilder::number(42.0) }
        ]);

        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "42;");
    }

    #[test]
    fn test_semicolon_strategy_remove() {
        let mut config = GeneratorConfig::default();
        config.semicolon = SemicolonStrategy::Remove;
        let generator = Generator::new(config);
        
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement { expression: AstTestBuilder::number(42.0) }
        ]);

        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "42");
    }

    /// Test generation diagnostics
    #[test]
    fn test_generation_diagnostics() {
        let config = GeneratorConfig::default();
        let generator = Generator::new(config);
        let original_source = "let x = 5;";
        
        let program = AstTestBuilder::program(vec![
            AstTestBuilder::var_declaration("x", Some(AstTestBuilder::number(5.0)), VariableDeclarationKind::Let)
        ]);

        let result = generator.generate(&program, Some(original_source)).unwrap();
        
        assert_eq!(result.diagnostics.original_size, original_source.len());
        assert_eq!(result.diagnostics.generated_size, result.code.len());
        assert!(result.diagnostics.compression_ratio >= 0.0);
        assert!(result.diagnostics.generation_time_ms >= 0.0);
    }

    /// Test this expression
    #[test]
    fn test_this_expression() {
        let config = GeneratorConfig::default();
        let generator = Generator::new(config);
        
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement { expression: Expression::ThisExpression }
        ]);

        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "this;");
    }

    /// Test CLI configuration creation
    #[test]
    fn test_cli_config_creation() {
        let config = GeneratorConfig::from_cli_args(
            "es5",
            "pretty",
            "always",
            "single",
            "license",
            "file"
        );
        
        assert!(matches!(config.ecma, crate::generator::EcmaScriptVersion::ES5));
        assert!(matches!(config.format, OutputFormat::Pretty));
        assert!(matches!(config.semicolon, SemicolonStrategy::Always));
        assert!(matches!(config.quote, QuoteStrategy::Single));
    }
}

/// Comprehensive ASI (Automatic Semicolon Insertion) hazard tests
/// Tests cover all edge cases from the specification to ensure proper
/// semicolon insertion and line terminator handling.
#[cfg(test)]
mod asi_hazard_tests {
    use super::*;

    /// Test return statement ASI hazards
    /// `return\nx` should become `return;x` or `return x;` depending on context
    #[test]
    fn test_return_statement_asi_hazards() {
        let config = GeneratorConfig::default();
        let generator = Generator::new(config);
        
        // Return with argument - should have space to prevent ASI
        let program = AstTestBuilder::program(vec![
            Statement::ReturnStatement {
                argument: Some(AstTestBuilder::id_expr("x"))
            }
        ]);
        
        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "return x;");
        
        // Return without argument - should be safe
        let program = AstTestBuilder::program(vec![
            Statement::ReturnStatement { argument: None }
        ]);
        
        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "return;");
    }

    /// Test statement continuation hazards
    /// Cases like `a\n(b)` should avoid being parsed as `a(b)`
    #[test]
    fn test_statement_continuation_hazards() {
        let config = GeneratorConfig::default();
        let generator = Generator::new(config);
        
        // Multiple statements that could be misinterpreted
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement { expression: AstTestBuilder::id_expr("a") },
            Statement::ExpressionStatement { 
                expression: Expression::CallExpression {
                    callee: Box::new(AstTestBuilder::id_expr("b")),
                    arguments: vec![]
                }
            }
        ]);
        
        let result = generator.generate(&program, None).unwrap();
        // Should have proper separation to prevent a(b) interpretation
        assert!(result.code.contains("a;") || result.code.contains("a\n"));
    }

    /// Test ASI with different semicolon strategies
    #[test]
    fn test_asi_with_semicolon_strategies() {
        // Test with "always" strategy
        let mut config = GeneratorConfig::default();
        config.semicolon = SemicolonStrategy::Always;
        let generator = Generator::new(config);
        
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement { expression: AstTestBuilder::number(42.0) }
        ]);
        
        let result = generator.generate(&program, None).unwrap();
        assert!(result.code.ends_with(";"));
        
        // Test with "remove" strategy
        let mut config = GeneratorConfig::default();
        config.semicolon = SemicolonStrategy::Remove;
        let generator = Generator::new(config);
        
        let result = generator.generate(&program, None).unwrap();
        assert!(!result.code.ends_with(";"));
    }

    /// Test ASI hazards with different output formats
    #[test]
    fn test_asi_with_output_formats() {
        // Compact format
        let mut config = GeneratorConfig::default();
        config.format = OutputFormat::Compact;
        let generator = Generator::new(config);
        
        let program = AstTestBuilder::program(vec![
            AstTestBuilder::var_declaration("x", Some(AstTestBuilder::number(1.0)), VariableDeclarationKind::Let),
            Statement::ExpressionStatement { expression: AstTestBuilder::id_expr("x") }
        ]);
        
        let result = generator.generate(&program, None).unwrap();
        // Should be compact but still ASI-safe
        assert!(result.code.len() < 20); // Compact
        assert!(result.code.contains("let x=1;x;") || result.code.contains("let x=1;x"));
        
        // Readable format
        let mut config = GeneratorConfig::default();
        config.format = OutputFormat::Readable;
        let generator = Generator::new(config);
        
        let result = generator.generate(&program, None).unwrap();
        // Should have proper spacing and line breaks
        assert!(result.code.contains(" "));
    }

    /// Test edge cases that could cause ASI issues
    #[test]
    fn test_asi_edge_cases() {
        let config = GeneratorConfig::default();
        let generator = Generator::new(config);
        
        // Test with this expression
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement { expression: Expression::ThisExpression }
        ]);
        
        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "this;");
        
        // Test with boolean literals
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement { expression: AstTestBuilder::boolean(true) },
            Statement::ExpressionStatement { expression: AstTestBuilder::boolean(false) }
        ]);
        
        let result = generator.generate(&program, None).unwrap();
        assert!(result.code.contains("true;") && result.code.contains("false;"));
    }
}

/// Comprehensive operator precedence tests
/// Tests cover all precedence levels and associativity rules to ensure
/// correct parentheses insertion and expression ordering.
#[cfg(test)]
mod precedence_tests {
    use super::*;

    /// Test arithmetic operator precedence
    /// Multiplication should bind tighter than addition
    #[test]
    fn test_arithmetic_precedence() {
        let config = GeneratorConfig::default();
        let generator = Generator::new(config);
        
        // a + b * c should not need parentheses around b * c
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement {
                expression: AstTestBuilder::binary_expr(
                    AstTestBuilder::id_expr("a"),
                    BinaryOperator::Add,
                    AstTestBuilder::binary_expr(
                        AstTestBuilder::id_expr("b"),
                        BinaryOperator::Multiply,
                        AstTestBuilder::id_expr("c")
                    )
                )
            }
        ]);
        
        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "a+b*c;");
        
        // (a + b) * c should need parentheses around a + b
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement {
                expression: AstTestBuilder::binary_expr(
                    AstTestBuilder::binary_expr(
                        AstTestBuilder::id_expr("a"),
                        BinaryOperator::Add,
                        AstTestBuilder::id_expr("b")
                    ),
                    BinaryOperator::Multiply,
                    AstTestBuilder::id_expr("c")
                )
            }
        ]);
        
        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "(a+b)*c;");
    }

    /// Test comparison operator precedence
    /// Arithmetic should bind tighter than comparison
    #[test]
    fn test_comparison_precedence() {
        let config = GeneratorConfig::default();
        let generator = Generator::new(config);
        
        // a + b < c + d should not need parentheses
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement {
                expression: AstTestBuilder::binary_expr(
                    AstTestBuilder::binary_expr(
                        AstTestBuilder::id_expr("a"),
                        BinaryOperator::Add,
                        AstTestBuilder::id_expr("b")
                    ),
                    BinaryOperator::LessThan,
                    AstTestBuilder::binary_expr(
                        AstTestBuilder::id_expr("c"),
                        BinaryOperator::Add,
                        AstTestBuilder::id_expr("d")
                    )
                )
            }
        ]);
        
        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "a+b<c+d;");
    }

    /// Test logical operator precedence
    /// && should bind tighter than ||
    #[test]
    fn test_logical_precedence() {
        let config = GeneratorConfig::default();
        let generator = Generator::new(config);
        
        // a && b || c && d should not need parentheses
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement {
                expression: AstTestBuilder::binary_expr(
                    AstTestBuilder::binary_expr(
                        AstTestBuilder::id_expr("a"),
                        BinaryOperator::LogicalAnd,
                        AstTestBuilder::id_expr("b")
                    ),
                    BinaryOperator::LogicalOr,
                    AstTestBuilder::binary_expr(
                        AstTestBuilder::id_expr("c"),
                        BinaryOperator::LogicalAnd,
                        AstTestBuilder::id_expr("d")
                    )
                )
            }
        ]);
        
        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "a&&b||c&&d;");
    }

    /// Test mixed precedence scenarios
    #[test]
    fn test_mixed_precedence() {
        let config = GeneratorConfig::default();
        let generator = Generator::new(config);
        
        // a + b * c == d should group as (a + (b * c)) == d
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement {
                expression: AstTestBuilder::binary_expr(
                    AstTestBuilder::binary_expr(
                        AstTestBuilder::id_expr("a"),
                        BinaryOperator::Add,
                        AstTestBuilder::binary_expr(
                            AstTestBuilder::id_expr("b"),
                            BinaryOperator::Multiply,
                            AstTestBuilder::id_expr("c")
                        )
                    ),
                    BinaryOperator::StrictEqual,
                    AstTestBuilder::id_expr("d")
                )
            }
        ]);
        
        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "a+b*c===d;");
    }

    /// Test precedence with parentheses preservation
    #[test]
    fn test_precedence_with_explicit_parentheses() {
        let config = GeneratorConfig::default();
        let generator = Generator::new(config);
        
        // Test that necessary parentheses are preserved
        // (a || b) && c should keep parentheses
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement {
                expression: AstTestBuilder::binary_expr(
                    AstTestBuilder::binary_expr(
                        AstTestBuilder::id_expr("a"),
                        BinaryOperator::LogicalOr,
                        AstTestBuilder::id_expr("b")
                    ),
                    BinaryOperator::LogicalAnd,
                    AstTestBuilder::id_expr("c")
                )
            }
        ]);
        
        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "(a||b)&&c;");
    }

    /// Test precedence edge cases
    #[test]
    fn test_precedence_edge_cases() {
        let config = GeneratorConfig::default();
        let generator = Generator::new(config);
        
        // Test deeply nested expressions
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement {
                expression: AstTestBuilder::binary_expr(
                    AstTestBuilder::binary_expr(
                        AstTestBuilder::binary_expr(
                            AstTestBuilder::id_expr("a"),
                            BinaryOperator::Add,
                            AstTestBuilder::id_expr("b")
                        ),
                        BinaryOperator::Multiply,
                        AstTestBuilder::id_expr("c")
                    ),
                    BinaryOperator::Subtract,
                    AstTestBuilder::id_expr("d")
                )
            }
        ]);
        
        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "(a+b)*c-d;");
    }
}

/// Comprehensive string and template literal tests
/// Tests cover quote selection, escape handling, and template literal processing
/// following the generator specification for optimal string representation.
#[cfg(test)]
mod string_tests {
    use super::*;

    /// Helper function to create a template literal
    fn template_literal(quasis: Vec<TemplateElement>, expressions: Vec<Expression>) -> Expression {
        Expression::TemplateLiteral { quasis, expressions }
    }

    /// Helper function to create a template element
    fn template_element(value: &str, tail: bool) -> TemplateElement {
        TemplateElement {
            value: value.to_string(),
            tail,
        }
    }

    /// Test quote selection for string literals
    /// Should choose the quote type that minimizes escaping
    #[test]
    fn test_quote_selection() {
        let config = GeneratorConfig::default();
        let generator = Generator::new(config);
        
        // String with single quotes - should use double quotes
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement { 
                expression: AstTestBuilder::string("don't")
            }
        ]);
        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "\"don't\";");
        
        // String with double quotes - should use single quotes
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement { 
                expression: AstTestBuilder::string("He said \"hello\"")
            }
        ]);
        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "'He said \"hello\"';");
        
        // String with both quotes - should choose better option
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement { 
                expression: AstTestBuilder::string("It's a \"test\"")
            }
        ]);
        let result = generator.generate(&program, None).unwrap();
        // Should choose single quotes to minimize escaping (1 escape vs 2 escapes)
        assert_eq!(result.code, "'It\\'s a \"test\"';");
    }

    /// Test escape sequence handling
    #[test]
    fn test_escape_sequences() {
        let config = GeneratorConfig::default();
        let generator = Generator::new(config);
        
        // Newline characters
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement { 
                expression: AstTestBuilder::string("line1\nline2")
            }
        ]);
        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "'line1\\nline2';");
        
        // Tab characters
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement { 
                expression: AstTestBuilder::string("col1\tcol2")
            }
        ]);
        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "'col1\\tcol2';");
        
        // Backslash
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement { 
                expression: AstTestBuilder::string("path\\to\\file")
            }
        ]);
        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "'path\\\\to\\\\file';");
    }

    /// Test Unicode and special character handling
    #[test]
    fn test_unicode_characters() {
        let config = GeneratorConfig::default();
        let generator = Generator::new(config);
        
        // Unicode characters (should be preserved as-is in most cases)
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement { 
                expression: AstTestBuilder::string("Hello 世界")
            }
        ]);
        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "'Hello 世界';");
        
        // Emoji characters
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement { 
                expression: AstTestBuilder::string("Test 🚀 emoji")
            }
        ]);
        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "'Test 🚀 emoji';");
    }

    /// Test empty and whitespace strings
    #[test]
    fn test_empty_and_whitespace_strings() {
        let config = GeneratorConfig::default();
        let generator = Generator::new(config);
        
        // Empty string
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement { 
                expression: AstTestBuilder::string("")
            }
        ]);
        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "'';");
        
        // Whitespace-only string
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement { 
                expression: AstTestBuilder::string("   ")
            }
        ]);
        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "'   ';");
        
        // String with various whitespace
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement { 
                expression: AstTestBuilder::string(" \t\n ")
            }
        ]);
        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "' \\t\\n ';");
    }

    /// Test template literal without expressions
    #[test]
    fn test_simple_template_literal() {
        let config = GeneratorConfig::default();
        let generator = Generator::new(config);
        
        // Simple template literal (no expressions)
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement {
                expression: template_literal(
                    vec![template_element("Hello world", true)],
                    vec![]
                )
            }
        ]);
        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "`Hello world`;");
    }

    /// Test template literal with expressions
    #[test]
    fn test_template_literal_with_expressions() {
        let config = GeneratorConfig::default();
        let generator = Generator::new(config);
        
        // Template literal with one expression
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement {
                expression: template_literal(
                    vec![
                        template_element("Hello ", false),
                        template_element("!", true)
                    ],
                    vec![AstTestBuilder::id_expr("name")]
                )
            }
        ]);
        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "`Hello ${name}!`;");
        
        // Template literal with multiple expressions
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement {
                expression: template_literal(
                    vec![
                        template_element("Value: ", false),
                        template_element(", Count: ", false),
                        template_element("", true)
                    ],
                    vec![
                        AstTestBuilder::id_expr("value"),
                        AstTestBuilder::id_expr("count")
                    ]
                )
            }
        ]);
        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "`Value: ${value}, Count: ${count}`;");
    }

    /// Test template literal with escape sequences
    #[test]
    fn test_template_literal_escapes() {
        let config = GeneratorConfig::default();
        let generator = Generator::new(config);
        
        // Template literal with backticks (should be escaped)
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement {
                expression: template_literal(
                    vec![template_element("Code: `example`", true)],
                    vec![]
                )
            }
        ]);
        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "`Code: \\`example\\``;");
        
        // Template literal with ${ sequences (should be escaped)
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement {
                expression: template_literal(
                    vec![template_element("Literal ${", true)],
                    vec![]
                )
            }
        ]);
        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "`Literal \\${`;");
    }

    /// Test template literal vs string literal optimization
    #[test]
    fn test_template_vs_string_optimization() {
        let config = GeneratorConfig::default();
        let generator = Generator::new(config);
        
        // Template literal with no expressions should potentially
        // be converted to regular string for efficiency
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement {
                expression: template_literal(
                    vec![template_element("simple", true)],
                    vec![]
                )
            }
        ]);
        let result = generator.generate(&program, None).unwrap();
        // For now, keep as template literal (could be optimized to 'simple' later)
        assert_eq!(result.code, "`simple`;");
    }

    /// Test complex template literal scenarios
    #[test]
    fn test_complex_template_scenarios() {
        let config = GeneratorConfig::default();
        let generator = Generator::new(config);
        
        // Nested expressions in template literal
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement {
                expression: template_literal(
                    vec![
                        template_element("Result: ", false),
                        template_element("", true)
                    ],
                    vec![
                        AstTestBuilder::binary_expr(
                            AstTestBuilder::id_expr("a"),
                            BinaryOperator::Add,
                            AstTestBuilder::id_expr("b")
                        )
                    ]
                )
            }
        ]);
        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "`Result: ${a+b}`;");
    }

    /// Test different quote strategies
    #[test]
    fn test_quote_strategies() {
        // Test Auto strategy (default)
        let config = GeneratorConfig {
            quote: QuoteStrategy::Auto,
            ..GeneratorConfig::default()
        };
        let generator = Generator::new(config);
        
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement { 
                expression: AstTestBuilder::string("test")
            }
        ]);
        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "'test';");
        
        // Test Single strategy
        let config = GeneratorConfig {
            quote: QuoteStrategy::Single,
            ..GeneratorConfig::default()
        };
        let generator = Generator::new(config);
        
        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "'test';");
        
        // Test Double strategy
        let config = GeneratorConfig {
            quote: QuoteStrategy::Double,
            ..GeneratorConfig::default()
        };
        let generator = Generator::new(config);
        
        let result = generator.generate(&program, None).unwrap();
        assert_eq!(result.code, "\"test\";");
    }

    /// Test string concatenation optimization opportunities
    #[test]
    fn test_string_optimization_opportunities() {
        let config = GeneratorConfig::default();
        let generator = Generator::new(config);
        
        // String concatenation that could be optimized
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement {
                expression: AstTestBuilder::binary_expr(
                    AstTestBuilder::string("hello "),
                    BinaryOperator::Add,
                    AstTestBuilder::string("world")
                )
            }
        ]);
        let result = generator.generate(&program, None).unwrap();
        // For now, generate as-is (optimization could happen in transformer)
        assert_eq!(result.code, "'hello '+'world';");
    }
}

/// Comprehensive source map tests
/// Tests cover mapping generation, position tracking, multi-file support,
/// and Source Maps V3 specification compliance.
#[cfg(test)]
mod sourcemap_tests {
    use super::*;
    use crate::generator::source_maps::*;
    use crate::generator::{SourceMapMode};

    /// Test basic source map generation
    #[test]
    fn test_basic_source_map_generation() {
        let config = GeneratorConfig {
            source_map: SourceMapMode::File,
            ..GeneratorConfig::default()
        };
        let generator = Generator::new(config);
        
        let program = AstTestBuilder::program(vec![
            AstTestBuilder::var_declaration("x", Some(AstTestBuilder::number(5.0)), VariableDeclarationKind::Let)
        ]);
        
        let source_file = Some("test.js");
        let result = generator.generate(&program, source_file).unwrap();
        
        assert!(result.source_map.is_some());
        let source_map = result.source_map.unwrap();
        assert_eq!(source_map.version, 3);
        assert_eq!(source_map.sources.len(), 1);
        assert_eq!(source_map.sources[0], "test.js");
    }

    /// Test source map without source file
    #[test]
    fn test_source_map_no_source_file() {
        let config = GeneratorConfig {
            source_map: SourceMapMode::File,
            ..GeneratorConfig::default()
        };
        let generator = Generator::new(config);
        
        let program = AstTestBuilder::program(vec![
            AstTestBuilder::var_declaration("x", Some(AstTestBuilder::number(5.0)), VariableDeclarationKind::Let)
        ]);
        
        let result = generator.generate(&program, None).unwrap();
        
        assert!(result.source_map.is_some());
        let source_map = result.source_map.unwrap();
        assert_eq!(source_map.sources.len(), 1);
        assert_eq!(source_map.sources[0], "<unknown>");
    }

    /// Test source map disabled
    #[test]
    fn test_source_map_disabled() {
        let config = GeneratorConfig {
            source_map: SourceMapMode::None,
            ..GeneratorConfig::default()
        };
        let generator = Generator::new(config);
        
        let program = AstTestBuilder::program(vec![
            AstTestBuilder::var_declaration("x", Some(AstTestBuilder::number(5.0)), VariableDeclarationKind::Let)
        ]);
        
        let result = generator.generate(&program, Some("test.js")).unwrap();
        assert!(result.source_map.is_none());
    }

    /// Test inline source map generation
    #[test]
    fn test_inline_source_map() {
        let config = GeneratorConfig {
            source_map: SourceMapMode::Inline,
            ..GeneratorConfig::default()
        };
        let generator = Generator::new(config);
        
        let program = AstTestBuilder::program(vec![
            AstTestBuilder::var_declaration("x", Some(AstTestBuilder::number(5.0)), VariableDeclarationKind::Let)
        ]);
        
        let result = generator.generate(&program, Some("test.js")).unwrap();
        assert!(result.source_map.is_some());
    }

    /// Test position mapping accuracy
    #[test]
    fn test_position_mapping_accuracy() {
        let config = GeneratorConfig {
            source_map: SourceMapMode::File,
            format: crate::generator::OutputFormat::Pretty,
            ..GeneratorConfig::default()
        };
        let generator = Generator::new(config);
        
        // Multi-statement program to test position tracking
        let program = AstTestBuilder::program(vec![
            AstTestBuilder::var_declaration("a", Some(AstTestBuilder::number(1.0)), VariableDeclarationKind::Let),
            AstTestBuilder::var_declaration("b", Some(AstTestBuilder::number(2.0)), VariableDeclarationKind::Const)
        ]);
        
        let result = generator.generate(&program, Some("multi.js")).unwrap();
        
        assert!(result.source_map.is_some());
        let source_map = result.source_map.unwrap();
        
        // Check that mappings were created
        assert!(!source_map.mappings.is_empty());
        
        // Verify source tracking
        assert_eq!(source_map.sources.len(), 1);
        assert_eq!(source_map.sources[0], "multi.js");
    }

    /// Test multi-file source map support
    #[test]
    fn test_multi_file_source_map() {
        let mut builder = SourceMapBuilder::new();
        
        // Add multiple source files
        let file1_idx = builder.add_source("file1.js");
        let file2_idx = builder.add_source("file2.js");
        let file3_idx = builder.add_source("file3.js");
        
        assert_eq!(file1_idx, 0);
        assert_eq!(file2_idx, 1);
        assert_eq!(file3_idx, 2);
        
        // Add mappings from different files
        builder.add_mapping(Mapping {
            generated: Position { line: 0, column: 0 },
            original: Some(Position { line: 5, column: 10 }),
            source_index: Some(file1_idx),
            name_index: None,
        });
        
        builder.add_mapping(Mapping {
            generated: Position { line: 1, column: 0 },
            original: Some(Position { line: 2, column: 5 }),
            source_index: Some(file2_idx),
            name_index: None,
        });
        
        let source_map = builder.build();
        
        assert_eq!(source_map.sources.len(), 3);
        assert_eq!(source_map.sources, vec!["file1.js", "file2.js", "file3.js"]);
        // Mappings are encoded as a string in the built source map
        assert!(!source_map.mappings.is_empty());
    }

    /// Test name tracking in source maps
    #[test]
    fn test_name_tracking() {
        let mut builder = SourceMapBuilder::new();
        
        let source_idx = builder.add_source("test.js");
        let name1_idx = builder.add_name("originalName");
        let name2_idx = builder.add_name("anotherName");
        
        assert_eq!(name1_idx, 0);
        assert_eq!(name2_idx, 1);
        
        // Add mapping with name
        builder.add_mapping(Mapping {
            generated: Position { line: 0, column: 0 },
            original: Some(Position { line: 1, column: 5 }),
            source_index: Some(source_idx),
            name_index: Some(name1_idx),
        });
        
        let source_map = builder.build();
        
        assert_eq!(source_map.names.len(), 2);
        assert_eq!(source_map.names, vec!["originalName", "anotherName"]);
    }

    /// Test source map JSON serialization
    #[test]
    fn test_source_map_json_serialization() {
        let mut builder = SourceMapBuilder::new();
        let source_idx = builder.add_source("test.js");
        let name_idx = builder.add_name("testVar");
        
        builder.add_mapping(Mapping {
            generated: Position { line: 0, column: 5 },
            original: Some(Position { line: 2, column: 10 }),
            source_index: Some(source_idx),
            name_index: Some(name_idx),
        });
        
        let source_map = builder.build();
        let json = source_map.to_json().unwrap();
        
        // Check that JSON contains required fields
        assert!(json.contains("\"version\":3"));
        assert!(json.contains("\"sources\":[\"test.js\"]"));
        assert!(json.contains("\"names\":[\"testVar\"]"));
        assert!(json.contains("\"mappings\":"));
    }

    /// Test source map data URL generation
    #[test]
    fn test_source_map_data_url() {
        let mut builder = SourceMapBuilder::new();
        builder.add_source("test.js");
        
        let source_map = builder.build();
        let data_url = source_map.to_inline_data_url().unwrap();
        
        assert!(data_url.starts_with("data:application/json;charset=utf-8;base64,"));
        
        // The data URL should contain encoded JSON
        let base64_part = &data_url["data:application/json;charset=utf-8;base64,".len()..];
        assert!(!base64_part.is_empty());
    }

    /// Test source mapping URL comment generation
    #[test]
    fn test_source_mapping_url_comment() {
        let source_map = SourceMap::new();
        
        let comment = source_map.add_source_mapping_url_comment("output.js.map");
        assert_eq!(comment, "//# sourceMappingURL=output.js.map");
        
        let data_comment = source_map.add_source_mapping_url_comment("data:application/json;base64,...");
        assert_eq!(data_comment, "//# sourceMappingURL=data:application/json;base64,...");
    }

    /// Test mapping segment creation and validation
    #[test]
    fn test_mapping_segment_creation() {
        let segment = MappingSegment {
            generated_column: 10,
            source_index: Some(0),
            original_line: Some(5),
            original_column: Some(15),
            name_index: Some(2),
        };
        
        assert_eq!(segment.generated_column, 10);
        assert_eq!(segment.source_index, Some(0));
        assert_eq!(segment.original_line, Some(5));
        assert_eq!(segment.original_column, Some(15));
        assert_eq!(segment.name_index, Some(2));
    }

    /// Test position tracking
    #[test]
    fn test_position_tracking() {
        let pos1 = Position { line: 0, column: 0 };
        let pos2 = Position { line: 10, column: 25 };
        
        assert_eq!(pos1.line, 0);
        assert_eq!(pos1.column, 0);
        assert_eq!(pos2.line, 10);
        assert_eq!(pos2.column, 25);
        
        assert_ne!(pos1, pos2);
    }

    /// Test empty source map
    #[test]
    fn test_empty_source_map() {
        let builder = SourceMapBuilder::new();
        let source_map = builder.build();
        
        assert_eq!(source_map.version, 3);
        assert!(source_map.sources.is_empty());
        assert!(source_map.names.is_empty());
        assert_eq!(source_map.mappings, "");
    }

    /// Test source map integration with generator
    #[test]
    fn test_source_map_generator_integration() {
        let config = GeneratorConfig {
            source_map: SourceMapMode::File,
            format: crate::generator::OutputFormat::Compact,
            ..GeneratorConfig::default()
        };
        let generator = Generator::new(config);
        
        // Complex program with multiple statements
        let program = AstTestBuilder::program(vec![
            AstTestBuilder::var_declaration(
                "result", 
                Some(AstTestBuilder::binary_expr(
                    AstTestBuilder::number(1.0),
                    BinaryOperator::Add,
                    AstTestBuilder::number(2.0)
                )), 
                VariableDeclarationKind::Const
            ),
            Statement::ExpressionStatement {
                expression: AstTestBuilder::binary_expr(
                    AstTestBuilder::id_expr("result"),
                    BinaryOperator::Multiply,
                    AstTestBuilder::number(3.0)
                )
            }
        ]);
        
        let result = generator.generate(&program, Some("complex.js")).unwrap();
        
        // Verify code generation
        assert!(result.code.contains("const result"));
        assert!(result.code.contains("result*3"));
        
        // Verify source map generation
        assert!(result.source_map.is_some());
        let source_map = result.source_map.unwrap();
        assert_eq!(source_map.sources[0], "complex.js");
        
        // Should have mappings for the generated code
        assert!(!source_map.mappings.is_empty());
    }

    /// Test source content inclusion
    #[test]
    fn test_source_content_inclusion() {
        let config = GeneratorConfig {
            source_map: SourceMapMode::File,
            include_sources_content: true,
            ..GeneratorConfig::default()
        };
        let generator = Generator::new(config);
        
        let program = AstTestBuilder::program(vec![
            AstTestBuilder::var_declaration("x", Some(AstTestBuilder::number(1.0)), VariableDeclarationKind::Let)
        ]);
        
        let result = generator.generate(&program, Some("with_content.js")).unwrap();
        
        assert!(result.source_map.is_some());
        let source_map = result.source_map.unwrap();
        
        // Note: source content might be None if not provided during generation
        // This tests the configuration is respected
        assert_eq!(source_map.sources[0], "with_content.js");
    }
}

/// Comprehensive error handling tests
/// Tests cover malformed AST validation, generation failures, and error recovery.
#[cfg(test)]
mod error_handling_tests {
    use super::*;
    use crate::generator::{GeneratorError, GeneratorResult};

    /// Test empty variable declaration error
    #[test]
    fn test_empty_variable_declaration_error() {
        let config = GeneratorConfig::default();
        let generator = Generator::new(config);
        
        // Create invalid variable declaration with no declarators
        let program = Program {
            body: vec![Statement::VariableDeclaration {
                declarations: vec![], // Empty declarations should cause error
                kind: VariableDeclarationKind::Let,
            }],
            source_type: ProgramSourceType::Script,
        };
        
        let result = generator.generate(&program, None);
        assert!(result.is_err());
        
        if let Err(GeneratorError::MissingRequiredField { field, node_type }) = result {
            assert_eq!(field, "declarations");
            assert_eq!(node_type, "VariableDeclaration");
        } else {
            panic!("Expected MissingRequiredField error");
        }
    }

    /// Test empty identifier error
    #[test]
    fn test_empty_identifier_error() {
        let config = GeneratorConfig::default();
        let generator = Generator::new(config);
        
        // Create invalid identifier with empty name
        let program = AstTestBuilder::program(vec![
            Statement::VariableDeclaration {
                declarations: vec![VariableDeclarator {
                    id: Pattern::Identifier(Identifier { name: String::new() }), // Empty name
                    init: Some(AstTestBuilder::number(1.0)),
                }],
                kind: VariableDeclarationKind::Let,
            }
        ]);
        
        let result = generator.generate(&program, None);
        assert!(result.is_err());
        
        if let Err(GeneratorError::IdentifierError { message, identifier }) = result {
            assert!(message.contains("cannot be empty"));
            assert_eq!(identifier, "<empty>");
        } else {
            panic!("Expected IdentifierError for empty identifier");
        }
    }

    /// Test invalid identifier start character
    #[test]
    fn test_invalid_identifier_start() {
        let config = GeneratorConfig::default();
        let generator = Generator::new(config);
        
        // Create identifier starting with number
        let program = AstTestBuilder::program(vec![
            Statement::VariableDeclaration {
                declarations: vec![VariableDeclarator {
                    id: Pattern::Identifier(Identifier { name: "123invalid".to_string() }),
                    init: Some(AstTestBuilder::number(1.0)),
                }],
                kind: VariableDeclarationKind::Let,
            }
        ]);
        
        let result = generator.generate(&program, None);
        assert!(result.is_err());
        
        if let Err(GeneratorError::IdentifierError { message, identifier }) = result {
            assert!(message.contains("must start with letter"));
            assert_eq!(identifier, "123invalid");
        } else {
            panic!("Expected IdentifierError for invalid identifier start");
        }
    }

    /// Test string with null character error
    #[test]
    fn test_string_null_character_error() {
        let config = GeneratorConfig::default();
        let generator = Generator::new(config);
        
        // Create string with null character
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement {
                expression: Expression::Literal(Literal::String(StringLiteral {
                    value: "test\0null".to_string(),
                }))
            }
        ]);
        
        let result = generator.generate(&program, None);
        assert!(result.is_err());
        
        if let Err(GeneratorError::StringProcessingError { message, content }) = result {
            assert!(message.contains("null character"));
            assert_eq!(content, "test\0null");
        } else {
            panic!("Expected StringProcessingError for null character");
        }
    }

    /// Test NaN number error
    #[test]
    fn test_nan_number_error() {
        let config = GeneratorConfig::default();
        let generator = Generator::new(config);
        
        // Create number with NaN value
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement {
                expression: Expression::Literal(Literal::Number(NumberLiteral {
                    value: f64::NAN,
                }))
            }
        ]);
        
        let result = generator.generate(&program, None);
        assert!(result.is_err());
        
        if let Err(GeneratorError::NumericValueError { message, value }) = result {
            assert!(message.contains("NaN"));
            assert_eq!(value, "NaN");
        } else {
            panic!("Expected NumericValueError for NaN");
        }
    }

    /// Test infinite number error
    #[test]
    fn test_infinite_number_error() {
        let config = GeneratorConfig::default();
        let generator = Generator::new(config);
        
        // Create number with infinite value
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement {
                expression: Expression::Literal(Literal::Number(NumberLiteral {
                    value: f64::INFINITY,
                }))
            }
        ]);
        
        let result = generator.generate(&program, None);
        assert!(result.is_err());
        
        if let Err(GeneratorError::NumericValueError { message, value }) = result {
            assert!(message.contains("Infinite"));
            assert_eq!(value, "inf");
        } else {
            panic!("Expected NumericValueError for infinity");
        }
    }

    /// Test empty regex pattern error
    #[test]
    fn test_empty_regex_pattern_error() {
        let config = GeneratorConfig::default();
        let generator = Generator::new(config);
        
        // Create regex with empty pattern
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement {
                expression: Expression::Literal(Literal::RegExp(RegExpLiteral {
                    pattern: String::new(), // Empty pattern
                    flags: "g".to_string(),
                }))
            }
        ]);
        
        let result = generator.generate(&program, None);
        assert!(result.is_err());
        
        if let Err(GeneratorError::StringProcessingError { message, content }) = result {
            assert!(message.contains("cannot be empty"));
            assert_eq!(content, "");
        } else {
            panic!("Expected StringProcessingError for empty regex pattern");
        }
    }

    /// Test template literal structure validation
    #[test]
    fn test_template_literal_structure_error() {
        let config = GeneratorConfig::default();
        let generator = Generator::new(config);
        
        // Create invalid template literal with mismatched quasis/expressions
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement {
                expression: Expression::TemplateLiteral {
                    quasis: vec![
                        TemplateElement { value: "Hello ".to_string(), tail: false },
                        TemplateElement { value: " world".to_string(), tail: true },
                    ],
                    expressions: vec![
                        AstTestBuilder::id_expr("name"),
                        AstTestBuilder::id_expr("extra"), // Extra expression
                    ],
                }
            }
        ]);
        
        let result = generator.generate(&program, None);
        assert!(result.is_err());
        
        if let Err(GeneratorError::TemplateLiteralError { message, template }) = result {
            assert!(message.contains("mismatch"));
            assert!(template.contains("quasis: 2"));
            assert!(template.contains("expressions: 2"));
        } else {
            panic!("Expected TemplateLiteralError for structure mismatch");
        }
    }

    /// Test empty template literal quasis error
    #[test]
    fn test_empty_template_literal_quasis() {
        let config = GeneratorConfig::default();
        let generator = Generator::new(config);
        
        // Create template literal with no quasis
        let program = AstTestBuilder::program(vec![
            Statement::ExpressionStatement {
                expression: Expression::TemplateLiteral {
                    quasis: vec![], // Empty quasis
                    expressions: vec![],
                }
            }
        ]);
        
        let result = generator.generate(&program, None);
        assert!(result.is_err());
        
        if let Err(GeneratorError::MissingRequiredField { field, node_type }) = result {
            assert_eq!(field, "quasis");
            assert_eq!(node_type, "TemplateLiteral");
        } else {
            panic!("Expected MissingRequiredField error for empty quasis");
        }
    }

    /// Test successful error recovery
    #[test]
    fn test_successful_generation_after_validation() {
        let config = GeneratorConfig::default();
        let generator = Generator::new(config);
        
        // Create valid program
        let program = AstTestBuilder::program(vec![
            AstTestBuilder::var_declaration("x", Some(AstTestBuilder::number(5.0)), VariableDeclarationKind::Let)
        ]);
        
        let result = generator.generate(&program, None);
        assert!(result.is_ok());
        
        let generation_result = result.unwrap();
        assert_eq!(generation_result.code, "let x=5;");
        assert_eq!(generation_result.diagnostics.warning_count, 0);
    }

    /// Test memory limit validation
    #[test]
    fn test_memory_limits() {
        // This test is more conceptual since we can't easily create a 10MB+ output
        // in a unit test, but we can verify the validation logic exists
        let config = GeneratorConfig::default();
        let generator = Generator::new(config);
        
        // Large but reasonable program should work
        let statements: Vec<Statement> = (0..100)
            .map(|i| {
                AstTestBuilder::var_declaration(
                    &format!("var{}", i),
                    Some(AstTestBuilder::number(i as f64)),
                    VariableDeclarationKind::Let
                )
            })
            .collect();
        
        let program = AstTestBuilder::program(statements);
        let result = generator.generate(&program, None);
        
        // Should succeed - 100 variable declarations shouldn't exceed memory limits
        assert!(result.is_ok());
    }

    /// Test error message formatting
    #[test]
    fn test_error_message_formatting() {
        let error = GeneratorError::MalformedAst {
            message: "Test error".to_string(),
            node_type: "TestNode".to_string(),
        };
        
        let error_string = format!("{}", error);
        assert!(error_string.contains("Malformed AST"));
        assert!(error_string.contains("Test error"));
        assert!(error_string.contains("TestNode"));
    }

    /// Test nested error propagation
    #[test]
    fn test_nested_error_propagation() {
        let config = GeneratorConfig::default();
        let generator = Generator::new(config);
        
        // Create nested structure with error deep inside
        let program = AstTestBuilder::program(vec![
            Statement::BlockStatement {
                body: vec![
                    Statement::VariableDeclaration {
                        declarations: vec![VariableDeclarator {
                            id: Pattern::Identifier(Identifier { name: String::new() }), // Error here
                            init: Some(AstTestBuilder::number(1.0)),
                        }],
                        kind: VariableDeclarationKind::Let,
                    }
                ],
            }
        ]);
        
        let result = generator.generate(&program, None);
        assert!(result.is_err());
        
        // Error should propagate up with context
        if let Err(GeneratorError::IdentifierError { .. }) = result {
            // Correct error type propagated
        } else {
            panic!("Expected IdentifierError to propagate from nested structure");
        }
    }
}