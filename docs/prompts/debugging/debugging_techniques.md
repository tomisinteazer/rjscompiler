# Rust Debugging Techniques and Troubleshooting

## Debugging Workflow

### Step-by-Step Debugging Process
1. **Reproduce the issue**: Create minimal test case
2. **Add logging**: Insert strategic debug prints or tracing
3. **Use debugger**: Attach debugger for complex issues
4. **Check assumptions**: Validate assumptions with assertions
5. **Isolate components**: Test components in isolation
6. **Profile performance**: Use profiling tools for performance issues

### Common Debugging Scenarios

#### Memory Issues
```rust
// Use valgrind or similar tools for memory debugging
// Add debug assertions for memory safety
fn process_large_data(data: &[u8]) -> Vec<ProcessedItem> {
    debug_assert!(!data.is_empty(), "Input data should not be empty");
    debug_assert!(data.len() < 1_000_000, "Data size should be reasonable");
    
    let mut result = Vec::with_capacity(data.len() / 4); // Pre-allocate
    
    // Process data...
    
    debug_assert!(result.len() <= data.len(), "Result should not exceed input size");
    result
}
```

#### Concurrency Issues
```rust
use std::sync::{Arc, Mutex};
use tracing::{debug, span, Level};

pub fn concurrent_processing(data: Arc<Mutex<SharedData>>) {
    let _span = span!(Level::DEBUG, "concurrent_processing", 
                     thread_id = ?std::thread::current().id()).entered();
    
    debug!("Attempting to acquire lock");
    let mut shared = data.lock().unwrap();
    debug!("Lock acquired successfully");
    
    // Critical section
    shared.process();
    
    debug!("Processing completed, releasing lock");
    // Lock automatically released when `shared` goes out of scope
}
```

#### Parser Debugging
```rust
pub fn debug_parse_expression(input: &str) -> Result<Expression, ParseError> {
    debug!("Parsing expression: '{}'", input);
    
    let tokens = tokenize(input)?;
    debug!("Tokens: {:?}", tokens);
    
    let mut parser = Parser::new(tokens);
    let result = parser.parse_expression();
    
    match &result {
        Ok(expr) => debug!("Successfully parsed expression: {:?}", expr),
        Err(e) => debug!("Parse error: {:?} at position {}", e, parser.position()),
    }
    
    result
}
```

## Profiling and Performance Debugging

### Using Criterion for Benchmarking
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_minification(c: &mut Criterion) {
    let source_code = include_str!("../test_data/large_file.js");
    
    c.bench_function("minify large file", |b| {
        b.iter(|| {
            let result = minify_javascript(black_box(source_code));
            black_box(result)
        })
    });
}

criterion_group!(benches, benchmark_minification);
criterion_main!(benches);
```

### Memory Profiling
```rust
#[cfg(feature = "profiling")]
use jemallocator::Jemalloc;

#[cfg(feature = "profiling")]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

pub fn memory_intensive_operation(data: &[String]) -> Vec<ProcessedData> {
    #[cfg(feature = "profiling")]
    let _guard = pprof::ProfilerGuard::new(100).unwrap();
    
    // Memory-intensive processing
    let result = data.iter()
        .map(|s| expensive_processing(s))
        .collect();
    
    #[cfg(feature = "profiling")]
    if let Ok(report) = _guard.report().build() {
        println!("Memory usage report: {:?}", report);
    }
    
    result
}
```

## Debugging Tools and Techniques

### GDB Integration
```bash
# Compile with debug symbols
cargo build --bin rjs-compiler

# Run with GDB
gdb target/debug/rjs-compiler

# Set breakpoints
(gdb) break src/main.rs:42
(gdb) break parser::parse_function

# Run with arguments
(gdb) run --verbose test.js

# Examine variables
(gdb) print variable_name
(gdb) print *pointer_name
```

### LLDB for macOS
```bash
# Compile with debug symbols
cargo build --bin rjs-compiler

# Run with LLDB
lldb target/debug/rjs-compiler

# Set breakpoints
(lldb) breakpoint set --file main.rs --line 42
(lldb) breakpoint set --name parse_function

# Run with arguments
(lldb) run -- --verbose test.js

# Examine variables
(lldb) frame variable
(lldb) expression variable_name
```

### Tracing Integration
```rust
use tracing::{debug, error, info, instrument, span, warn, Level};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

pub fn setup_tracing() {
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .with_max_level(Level::DEBUG)
        .finish();
    
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set tracing subscriber");
}

#[instrument(skip(config))]
pub fn process_file(path: &Path, config: &Config) -> Result<String, ProcessingError> {
    info!("Processing file: {}", path.display());
    
    let content = std::fs::read_to_string(path)?;
    debug!("File size: {} bytes", content.len());
    
    let result = minify_content(&content, config)?;
    info!("Minification completed, reduced from {} to {} bytes", 
          content.len(), result.len());
    
    Ok(result)
}
```

## Test-Driven Debugging

### Writing Failing Tests First
```rust
#[cfg(test)]
mod debug_tests {
    use super::*;

    #[test]
    #[should_panic(expected = "Variable 'x' not found in scope")]
    fn should_detect_undefined_variable() {
        let source = "function test() { return x; }"; // 'x' is undefined
        let result = analyze_scope(source);
        // This test helps debug scope analysis issues
    }

    #[test]
    fn should_preserve_function_semantics() {
        let original = r#"
            function calculateSum(a, b) {
                return a + b;
            }
            console.log(calculateSum(5, 3));
        "#;
        
        let minified = minify_javascript(original).unwrap();
        
        // Both should produce the same result when executed
        assert_eq!(execute_javascript(original), execute_javascript(&minified));
    }
}
```

### Regression Testing
```rust
#[cfg(test)]
mod regression_tests {
    use super::*;

    #[test]
    fn issue_42_variable_shadowing() {
        // Regression test for GitHub issue #42
        let source = r#"
            function outer() {
                var x = 1;
                function inner() {
                    var x = 2; // This should shadow outer x
                    return x;
                }
                return inner() + x;
            }
        "#;
        
        let result = minify_javascript(source);
        assert!(result.is_ok());
        
        let minified = result.unwrap();
        // Ensure variable shadowing is preserved
        assert!(minified.contains("var a=1")); // outer x
        assert!(minified.contains("var a=2")); // inner x (should be different variable)
    }
}
```

## Debug Utilities and Helpers

### Debug Formatting Helpers
```rust
pub struct DebugAst<'a>(pub &'a AstNode);

impl<'a> std::fmt::Debug for DebugAst<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            AstNode::Function { name, params, body } => {
                write!(f, "Function({}, params: {:?}, body: [", name, params)?;
                for stmt in body {
                    write!(f, "{:?}, ", DebugAst(stmt))?;
                }
                write!(f, "])")
            }
            AstNode::Variable { name, value } => {
                write!(f, "Var({} = {:?})", name, DebugAst(value))
            }
            // ... other node types
        }
    }
}

// Usage in debugging
pub fn debug_ast_structure(ast: &AstNode) {
    println!("AST Structure: {:#?}", DebugAst(ast));
}
```

### Conditional Debug Output
```rust
macro_rules! debug_minification {
    ($($arg:tt)*) => {
        if cfg!(feature = "debug-minification") {
            eprintln!("[MINIFY DEBUG] {}", format!($($arg)*));
        }
    };
}

pub fn rename_variable(old_name: &str, new_name: &str, scope: &Scope) {
    debug_minification!("Renaming '{}' to '{}' in scope {:p}", old_name, new_name, scope);
    
    // Perform renaming...
    
    debug_minification!("Renaming completed successfully");
}
```

### Interactive Debugging
```rust
#[cfg(debug_assertions)]
macro_rules! debug_pause {
    () => {
        println!("Debug pause at {}:{}", file!(), line!());
        println!("Press Enter to continue...");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
    };
}

pub fn complex_transformation(ast: &mut AstNode) {
    debug_pause!(); // Pause for inspection
    
    // Apply first transformation
    transform_step_1(ast);
    
    debug_pause!(); // Pause to check intermediate state
    
    // Apply second transformation
    transform_step_2(ast);
}
```

## Performance Debugging Patterns

### Timing Measurements
```rust
use std::time::Instant;

pub fn measure_performance<F, R>(operation_name: &str, f: F) -> R
where
    F: FnOnce() -> R,
{
    let start = Instant::now();
    let result = f();
    let duration = start.elapsed();
    
    println!("{} took: {:?}", operation_name, duration);
    result
}

// Usage
let minified = measure_performance("JavaScript minification", || {
    minify_javascript(&source_code)
});
```

### Memory Usage Tracking
```rust
#[cfg(feature = "memory-tracking")]
pub fn track_memory_usage<F, R>(operation_name: &str, f: F) -> R
where
    F: FnOnce() -> R,
{
    let initial_memory = get_memory_usage();
    let result = f();
    let final_memory = get_memory_usage();
    
    println!("{} memory delta: {} bytes", 
             operation_name, 
             final_memory.saturating_sub(initial_memory));
    result
}

#[cfg(feature = "memory-tracking")]
fn get_memory_usage() -> usize {
    // Platform-specific memory usage retrieval
    // Implementation depends on target platform
    0 // Placeholder
}
```