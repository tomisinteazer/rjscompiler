# Rust Testing Standards and TDD Guidelines

## Test Organization and Structure

### Unit Test Structure
Organize tests using the AAA (Arrange, Act, Assert) pattern:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_parse_simple_function_declaration() {
        // Arrange
        let source = "function greet(name) { return `Hello, ${name}!`; }";
        let mut parser = Parser::new(source);
        
        // Act
        let result = parser.parse_function();
        
        // Assert
        assert!(result.is_ok());
        let function = result.unwrap();
        assert_eq!(function.name(), "greet");
        assert_eq!(function.parameters().len(), 1);
        assert_eq!(function.parameters()[0].name(), "name");
    }

    #[test]
    fn should_return_error_for_malformed_function() {
        // Arrange
        let source = "function test( { return 42; }"; // Missing closing parenthesis
        let mut parser = Parser::new(source);
        
        // Act
        let result = parser.parse_function();
        
        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::ExpectedToken { expected, found, position } => {
                assert_eq!(expected, TokenKind::RightParen);
                assert_eq!(position.line, 1);
                assert_eq!(position.column, 15);
            }
            other => panic!("Expected ExpectedToken error, got: {:?}", other),
        }
    }
}
```

### Integration Test Organization
Structure integration tests for different components:

```rust
// tests/integration_test.rs
use rjs_compiler::{minify_javascript, MinifierConfig, OptimizationLevel};

#[test]
fn complete_minification_workflow() {
    let source = include_str!("fixtures/complex_example.js");
    let config = MinifierConfig::builder()
        .optimization_level(OptimizationLevel::Aggressive)
        .build()
        .unwrap();
    
    let result = minify_javascript(source, &config);
    
    assert!(result.is_ok());
    let minified = result.unwrap();
    
    // Verify size reduction
    assert!(minified.len() < source.len() * 0.5);
    
    // Verify functionality preservation
    assert!(verify_semantic_equivalence(source, &minified));
}

#[test]
fn batch_processing_preserves_individual_results() {
    let test_files = vec![
        "tests/fixtures/simple.js",
        "tests/fixtures/complex.js", 
        "tests/fixtures/edge_cases.js",
    ];
    
    let config = MinifierConfig::default();
    
    // Process individually
    let individual_results: Vec<_> = test_files
        .iter()
        .map(|path| {
            let source = std::fs::read_to_string(path).unwrap();
            minify_javascript(&source, &config).unwrap()
        })
        .collect();
    
    // Process as batch
    let batch_results = minify_multiple_files(&test_files, &config).unwrap();
    
    // Results should be identical
    assert_eq!(individual_results, batch_results);
}
```

## Test-Driven Development (TDD)

### Red-Green-Refactor Cycle
Example TDD implementation for variable renaming:

```rust
// Step 1: RED - Write failing test first
#[test]
fn should_rename_variables_to_short_names() {
    let source = r#"
        function calculateSum(firstNumber, secondNumber) {
            const result = firstNumber + secondNumber;
            return result;
        }
    "#;
    
    let config = MinifierConfig::builder()
        .optimization_level(OptimizationLevel::Aggressive)
        .build()
        .unwrap();
    
    let minified = minify_javascript(source, &config).unwrap();
    
    // Should rename variables to short names
    assert!(minified.contains("function a(b,c)"));
    assert!(minified.contains("const d=b+c"));
    assert!(!minified.contains("firstNumber"));
    assert!(!minified.contains("secondNumber"));
    assert!(!minified.contains("result"));
}

// Step 2: GREEN - Implement minimal code to pass
impl VariableRenamer {
    pub fn rename_variables(&mut self, ast: &mut AstNode) -> Result<(), RenameError> {
        // Minimal implementation to make test pass
        let mut name_generator = ShortNameGenerator::new();
        self.rename_in_node(ast, &mut name_generator)
    }
    
    fn rename_in_node(&mut self, node: &mut AstNode, generator: &mut ShortNameGenerator) -> Result<(), RenameError> {
        match node {
            AstNode::Variable { name, .. } => {
                *name = generator.next_name();
            }
            AstNode::Function { name, params, body } => {
                *name = generator.next_name();
                for param in params {
                    param.name = generator.next_name();
                }
                self.rename_in_node(body, generator)?;
            }
            // Handle other node types...
        }
        Ok(())
    }
}

// Step 3: REFACTOR - Improve implementation while keeping tests green
impl VariableRenamer {
    pub fn rename_variables(&mut self, ast: &mut AstNode) -> Result<(), RenameError> {
        // Build symbol table first for safe renaming
        let symbol_table = self.build_symbol_table(ast)?;
        let renaming_map = self.generate_safe_renaming(&symbol_table)?;
        self.apply_renaming(ast, &renaming_map)
    }
    
    fn build_symbol_table(&self, ast: &AstNode) -> Result<SymbolTable, RenameError> {
        let mut analyzer = ScopeAnalyzer::new();
        analyzer.analyze(ast)
    }
    
    fn generate_safe_renaming(&self, symbol_table: &SymbolTable) -> Result<RenamingMap, RenameError> {
        let mut map = RenamingMap::new();
        let mut name_generator = ShortNameGenerator::new();
        
        // Sort symbols by usage frequency for optimal naming
        let symbols_by_frequency = symbol_table.symbols_by_usage_frequency();
        
        for symbol in symbols_by_frequency {
            if self.can_safely_rename(&symbol) {
                map.insert(symbol.id(), name_generator.next_name());
            }
        }
        
        Ok(map)
    }
}
```

### Property-Based Testing
Use `proptest` for comprehensive edge case testing:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn minification_preserves_program_behavior(
        source in javascript_program_strategy(),
    ) {
        let config = MinifierConfig::default();
        
        // Property: minification should preserve program behavior
        if let Ok(minified) = minify_javascript(&source, &config) {
            prop_assert!(verify_semantic_equivalence(&source, &minified));
        }
    }
    
    #[test]
    fn minification_reduces_size(
        source in non_trivial_javascript_strategy(),
    ) {
        let config = MinifierConfig::builder()
            .optimization_level(OptimizationLevel::Aggressive)
            .build()
            .unwrap();
        
        // Property: aggressive minification should reduce size
        if let Ok(minified) = minify_javascript(&source, &config) {
            prop_assert!(minified.len() < source.len());
        }
    }
    
    #[test]
    fn parser_handles_arbitrary_input_gracefully(
        input in ".*",
    ) {
        let mut parser = Parser::new(&input);
        
        // Property: parser should never panic, regardless of input
        let result = parser.parse();
        
        // We don't care if it succeeds or fails, just that it doesn't panic
        drop(result);
    }
}

// Strategy for generating valid JavaScript programs
fn javascript_program_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        function_declaration_strategy(),
        variable_declaration_strategy(),
        expression_statement_strategy(),
    ]
}

fn function_declaration_strategy() -> impl Strategy<Value = String> {
    (identifier_strategy(), param_list_strategy(), block_statement_strategy())
        .prop_map(|(name, params, body)| {
            format!("function {}({}) {}", name, params.join(", "), body)
        })
}

fn identifier_strategy() -> impl Strategy<Value = String> {
    "[a-zA-Z_$][a-zA-Z0-9_$]*"
}
```

### Performance Testing
Benchmark critical paths and detect regressions:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

fn benchmark_minification_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("minification");
    
    for size in [1_000, 10_000, 100_000].iter() {
        let source = generate_javascript_source(*size);
        let config = MinifierConfig::default();
        
        group.bench_with_input(
            BenchmarkId::new("parse", size),
            &source,
            |b, source| {
                b.iter(|| {
                    let mut parser = Parser::new(black_box(source));
                    black_box(parser.parse().unwrap())
                })
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("minify", size),
            &source,
            |b, source| {
                b.iter(|| {
                    minify_javascript(black_box(source), black_box(&config)).unwrap()
                })
            },
        );
    }
    
    group.finish();
}

criterion_group!(benches, benchmark_minification_performance);
criterion_main!(benches);

// Performance regression tests
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn parsing_performance_regression() {
        let large_source = include_str!("fixtures/large_file.js");
        
        let start = Instant::now();
        let _ast = parse_javascript(large_source).unwrap();
        let duration = start.elapsed();
        
        // Fail if parsing takes longer than expected
        assert!(
            duration < Duration::from_millis(50),
            "Parsing regression detected: took {:?} (max: 50ms)",
            duration
        );
    }
    
    #[test]
    fn memory_usage_regression() {
        let source = generate_javascript_source(100_000);
        
        let initial_memory = get_memory_usage();
        let _minified = minify_javascript(&source, &MinifierConfig::default()).unwrap();
        let peak_memory = get_memory_usage();
        
        let memory_delta = peak_memory - initial_memory;
        let source_size = source.len();
        
        // Memory usage should not exceed 3x source size
        assert!(
            memory_delta < source_size * 3,
            "Memory usage regression: used {} bytes for {} byte input (ratio: {:.1}x)",
            memory_delta,
            source_size,
            memory_delta as f64 / source_size as f64
        );
    }
}
```

## Test Coverage and Quality

### Comprehensive Error Testing
Test all error conditions thoroughly:

```rust
#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[test]
    fn should_handle_syntax_errors_gracefully() {
        let test_cases = vec![
            ("function test(", "Missing closing parenthesis"),
            ("var x = ;", "Missing variable initializer"), 
            ("if (true { }", "Missing closing parenthesis in condition"),
            ("function() {}", "Missing function name"),
            ("let const = 5;", "Reserved keyword as identifier"),
        ];
        
        for (source, description) in test_cases {
            let result = parse_javascript(source);
            assert!(
                result.is_err(),
                "Expected error for case: {} ({})",
                source,
                description
            );
        }
    }
    
    #[test]
    fn should_provide_detailed_error_context() {
        let source = r#"
            function valid() {}
            function invalid( {
                return 42;
            }
            function alsoValid() {}
        "#;
        
        let result = parse_javascript(source);
        assert!(result.is_err());
        
        let error = result.unwrap_err();
        match error {
            ParseError::ExpectedToken { position, .. } => {
                assert_eq!(position.line, 3); // Error on line 3
                assert!(position.column > 0);
            }
            other => panic!("Expected ExpectedToken error, got: {:?}", other),
        }
    }
    
    #[test]
    fn should_recover_from_errors_in_batch_mode() {
        let sources = vec![
            "function valid1() { return 1; }",
            "function invalid( { return 2; }", // Error
            "function valid2() { return 3; }",
            "var broken = ;", // Another error
            "function valid3() { return 4; }",
        ];
        
        let results = parse_multiple_with_recovery(&sources);
        
        // Should recover 3 valid functions
        assert_eq!(results.successful.len(), 3);
        assert_eq!(results.errors.len(), 2);
        
        // Verify recovered functions
        let names: Vec<_> = results.successful
            .iter()
            .map(|ast| ast.function_name())
            .collect();
        assert_eq!(names, vec!["valid1", "valid2", "valid3"]);
    }
}
```

### Edge Case Testing
Test boundary conditions and edge cases:

```rust
#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[test]
    fn should_handle_deeply_nested_functions() {
        let mut source = String::new();
        let depth = 100;
        
        // Create deeply nested function structure
        for i in 0..depth {
            source.push_str(&format!("function f{}() {{ ", i));
        }
        source.push_str("return 42;");
        for _ in 0..depth {
            source.push_str(" }");
        }
        
        let result = minify_javascript(&source, &MinifierConfig::default());
        assert!(result.is_ok());
        
        let minified = result.unwrap();
        assert!(minified.len() < source.len());
    }
    
    #[test]
    fn should_handle_very_long_identifiers() {
        let long_name = "a".repeat(1000);
        let source = format!("function {}() {{ return 42; }}", long_name);
        
        let result = minify_javascript(&source, &MinifierConfig::default());
        assert!(result.is_ok());
        
        let minified = result.unwrap();
        // Long identifier should be renamed to short one
        assert!(!minified.contains(&long_name));
    }
    
    #[test]
    fn should_handle_unicode_identifiers() {
        let source = r#"
            function 测试函数() {
                const αβγ = 42;
                const 変数 = "test";
                return αβγ + 変数.length;
            }
        "#;
        
        let result = minify_javascript(source, &MinifierConfig::default());
        assert!(result.is_ok());
        
        // Unicode identifiers should be preserved or safely renamed
        let minified = result.unwrap();
        assert!(verify_semantic_equivalence(source, &minified));
    }
    
    #[test]
    fn should_handle_empty_and_whitespace_only_input() {
        let test_cases = vec![
            "",           // Empty
            "   ",        // Only spaces
            "\n\n\t\n",   // Only whitespace
            "//comment",  // Only comment
            "/* block comment */", // Only block comment
        ];
        
        for source in test_cases {
            let result = minify_javascript(source, &MinifierConfig::default());
            match result {
                Ok(minified) => assert!(minified.trim().is_empty()),
                Err(_) => {}, // Empty input might be considered an error
            }
        }
    }
}
```

### Test Data Management
Organize test fixtures and data:

```rust
// tests/common/mod.rs
use std::path::Path;

pub struct TestFixtures;

impl TestFixtures {
    pub fn load_javascript(name: &str) -> String {
        let path = Path::new("tests/fixtures/javascript").join(format!("{}.js", name));
        std::fs::read_to_string(path).unwrap()
    }
    
    pub fn load_expected_output(name: &str) -> String {
        let path = Path::new("tests/fixtures/expected").join(format!("{}.min.js", name));
        std::fs::read_to_string(path).unwrap()
    }
    
    pub fn all_test_cases() -> Vec<(&'static str, &'static str)> {
        vec![
            ("simple_function", "Basic function declaration"),
            ("variable_scoping", "Complex variable scoping"),
            ("arrow_functions", "ES6 arrow functions"),
            ("destructuring", "Destructuring assignment"),
            ("async_await", "Async/await patterns"),
            ("class_syntax", "ES6 class syntax"),
            ("template_literals", "Template string literals"),
            ("regex_edge_cases", "Regular expression edge cases"),
        ]
    }
}

// Usage in tests
#[test]
fn test_all_fixtures() {
    let config = MinifierConfig::default();
    
    for (name, description) in TestFixtures::all_test_cases() {
        let source = TestFixtures::load_javascript(name);
        let expected = TestFixtures::load_expected_output(name);
        
        let result = minify_javascript(&source, &config);
        assert!(
            result.is_ok(),
            "Failed to minify {} ({}): {:?}",
            name,
            description,
            result.unwrap_err()
        );
        
        let minified = result.unwrap();
        assert_eq!(
            minified.trim(),
            expected.trim(),
            "Output mismatch for {} ({})",
            name,
            description
        );
    }
}
```