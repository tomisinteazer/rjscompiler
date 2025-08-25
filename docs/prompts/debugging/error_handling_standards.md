# Rust Debugging Standards and Practices

## Error Handling for Debugging

### Custom Error Types
Use `thiserror` for creating comprehensive error types that aid in debugging:

```rust
#[derive(Debug, thiserror::Error)]
pub enum MinifierError {
    #[error("Parse error at line {line}, column {column}: {message}")]
    ParseError {
        line: usize,
        column: usize,
        message: String,
    },
    
    #[error("Scope analysis failed: {reason}")]
    ScopeAnalysisError { reason: String },
    
    #[error("Transformation failed on node {node_type}: {details}")]
    TransformationError {
        node_type: String,
        details: String,
    },
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Configuration error: {field} = {value}")]
    ConfigurationError { field: String, value: String },
}
```

### Error Propagation Strategies
- **Use `?` operator**: For clean error propagation
- **Context preservation**: Maintain error context through the call stack
- **Error wrapping**: Wrap lower-level errors with domain-specific context
- **Graceful degradation**: Handle recoverable errors without panicking

```rust
pub fn process_javascript_file(path: &Path) -> Result<String, MinifierError> {
    let source = std::fs::read_to_string(path)
        .map_err(|e| MinifierError::IoError(e))?;
    
    let ast = parse_javascript(&source)
        .map_err(|e| MinifierError::ParseError {
            line: e.line,
            column: e.column,
            message: e.message,
        })?;
    
    let analyzed_ast = analyze_scope(ast)
        .map_err(|e| MinifierError::ScopeAnalysisError {
            reason: e.to_string(),
        })?;
    
    let minified = transform_ast(analyzed_ast)?;
    let output = generate_code(minified)?;
    
    Ok(output)
}
```

## Debugging Best Practices

### Avoid Panics
- **Never panic on user input**: Always return `Result` for user-facing operations
- **Document panic conditions**: If panicking is necessary, document when and why
- **Use `expect()` with meaningful messages**: When unwrapping is safe, provide context

```rust
// Good: Handle user input gracefully
pub fn set_optimization_level(level: &str) -> Result<OptimizationLevel, ConfigurationError> {
    match level {
        "safe" => Ok(OptimizationLevel::Safe),
        "aggressive" => Ok(OptimizationLevel::Aggressive),
        _ => Err(ConfigurationError::InvalidLevel(level.to_string())),
    }
}

// Good: Documented panic for programmer errors
pub fn get_symbol_by_index(symbols: &[Symbol], index: usize) -> &Symbol {
    symbols.get(index)
        .expect("Symbol index should be validated before calling this function")
}

// Avoid: Panicking on user input
pub fn set_optimization_level_bad(level: &str) -> OptimizationLevel {
    match level {
        "safe" => OptimizationLevel::Safe,
        "aggressive" => OptimizationLevel::Aggressive,
        _ => panic!("Invalid optimization level: {}", level), // BAD!
    }
}
```

### Logging and Tracing
Use structured logging for debugging complex operations:

```rust
use tracing::{debug, error, info, span, warn, Level};

pub fn analyze_function_scope(func: &FunctionNode) -> Result<Scope, ScopeError> {
    let _span = span!(Level::DEBUG, "analyze_function_scope", 
                     function_name = %func.name).entered();
    
    info!("Starting scope analysis for function: {}", func.name);
    
    let mut scope = Scope::new();
    
    // Add parameters to scope
    for param in &func.parameters {
        debug!("Adding parameter to scope: {}", param.name);
        scope.add_symbol(param.name.clone(), SymbolType::Parameter);
    }
    
    // Analyze function body
    match analyze_block(&func.body, &mut scope) {
        Ok(_) => {
            info!("Scope analysis completed successfully for function: {}", func.name);
            Ok(scope)
        }
        Err(e) => {
            error!("Scope analysis failed for function {}: {}", func.name, e);
            Err(e)
        }
    }
}
```

### Debug Assertions
Use debug assertions for internal consistency checks:

```rust
pub fn add_symbol_to_scope(scope: &mut Scope, symbol: Symbol) {
    debug_assert!(!scope.contains(&symbol.name), 
                  "Symbol '{}' already exists in scope", symbol.name);
    debug_assert!(!symbol.name.is_empty(), 
                  "Symbol name cannot be empty");
    
    scope.symbols.insert(symbol.name.clone(), symbol);
}
```

## Error Context and Recovery

### Providing Context
Always provide sufficient context for debugging:

```rust
impl MinifierError {
    pub fn with_context(self, context: &str) -> Self {
        match self {
            MinifierError::ParseError { line, column, message } => {
                MinifierError::ParseError {
                    line,
                    column,
                    message: format!("{} (context: {})", message, context),
                }
            }
            other => other,
        }
    }
}

// Usage
let result = parse_expression(input)
    .map_err(|e| e.with_context("while parsing function arguments"))?;
```

### Error Recovery Strategies
Implement graceful error recovery where possible:

```rust
pub fn parse_statements(tokens: &[Token]) -> (Vec<Statement>, Vec<ParseError>) {
    let mut statements = Vec::new();
    let mut errors = Vec::new();
    let mut position = 0;
    
    while position < tokens.len() {
        match parse_statement(&tokens[position..]) {
            Ok((stmt, consumed)) => {
                statements.push(stmt);
                position += consumed;
            }
            Err(e) => {
                errors.push(e);
                // Try to recover by skipping to the next statement boundary
                position = find_next_statement_boundary(&tokens[position..])
                    .unwrap_or(tokens.len());
            }
        }
    }
    
    (statements, errors)
}
```

## Testing for Debugging

### Test Error Conditions
Always test error paths and edge cases:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_handle_invalid_javascript_syntax() {
        let invalid_js = "function test( {"; // Missing closing parenthesis
        let result = parse_javascript(invalid_js);
        
        assert!(result.is_err());
        match result.unwrap_err() {
            MinifierError::ParseError { line, column, .. } => {
                assert_eq!(line, 1);
                assert_eq!(column, 15);
            }
            other => panic!("Expected ParseError, got: {:?}", other),
        }
    }

    #[test]
    fn should_recover_from_single_statement_error() {
        let js_with_error = r#"
            function valid() {}
            function invalid( {  // Error here
            function alsoValid() {}
        "#;
        
        let (statements, errors) = parse_statements_with_recovery(js_with_error);
        
        assert_eq!(statements.len(), 2); // Should recover 2 valid functions
        assert_eq!(errors.len(), 1);     // Should capture 1 error
    }
}
```

### Property-Based Testing for Edge Cases
Use `proptest` to find edge cases:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn scope_analysis_should_not_panic(
        function_name in "[a-zA-Z][a-zA-Z0-9_]*",
        param_count in 0..10usize,
    ) {
        let mut func = FunctionNode::new(function_name);
        for i in 0..param_count {
            func.add_parameter(format!("param_{}", i));
        }
        
        // Should not panic regardless of input
        let result = analyze_function_scope(&func);
        // We don't care about success/failure, just that it doesn't panic
    }
}
```

## Development Debugging Tools

### Cargo Configuration for Debugging
```toml
[profile.dev]
debug = true
opt-level = 0
overflow-checks = true
lto = false

[profile.release]
debug = false
opt-level = 3
lto = true
panic = "abort"
```

### Environment Variables for Debug Control
```rust
pub fn setup_debugging() {
    if std::env::var("RJS_DEBUG").is_ok() {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    }
    
    if std::env::var("RJS_TRACE").is_ok() {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::TRACE)
            .init();
    }
}
```

## Common Debugging Patterns

### Debug Formatting
Implement helpful debug output:

```rust
#[derive(Debug)]
pub struct Scope {
    pub symbols: HashMap<String, Symbol>,
    pub parent: Option<Box<Scope>>,
}

impl std::fmt::Display for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Scope with {} symbols:", self.symbols.len())?;
        for (name, symbol) in &self.symbols {
            writeln!(f, "  {}: {:?}", name, symbol)?;
        }
        Ok(())
    }
}
```

### Conditional Debugging
```rust
macro_rules! debug_print {
    ($($arg:tt)*) => {
        if cfg!(debug_assertions) {
            eprintln!($($arg)*);
        }
    };
}

pub fn transform_variable_name(old_name: &str, new_name: &str) {
    debug_print!("Renaming variable: {} -> {}", old_name, new_name);
    // Transformation logic...
}
```