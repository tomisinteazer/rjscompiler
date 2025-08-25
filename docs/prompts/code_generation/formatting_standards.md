# Rust Code Formatting Standards

## Formatting Parameters

### Line Length and Spacing
- **Maximum line length**: 100 characters (rustfmt default)
- **Indentation**: 4 spaces (no tabs)
- **Trailing commas**: Use in multi-line expressions for cleaner diffs
- **Blank lines**: One blank line between items, two around module declarations

### Code Structure
```rust
// Use rustfmt with these key settings:
// max_width = 100
// hard_tabs = false
// tab_spaces = 4
// newline_style = "Unix"
// use_small_heuristics = "Default"
```

### Import Organization Rules
1. **Standard library first**: `std` and `core` imports
2. **External crates**: Third-party dependencies
3. **Local modules**: Current crate modules
4. **Group separation**: Blank line between groups
5. **Alphabetical order**: Within each group

#### Example Import Organization
```rust
use std::collections::HashMap;
use std::fs::File;

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::config::Config;
use crate::error::AppError;
```

## Rustfmt Configuration

### Basic Configuration
```toml
[tool.rustfmt]
max_width = 100
hard_tabs = false
tab_spaces = 4
newline_style = "Unix"
use_small_heuristics = "Default"
```

### Advanced Formatting Rules
- **Trailing commas**: Always use in multi-line expressions
- **Function calls**: Break long function calls across multiple lines
- **Struct formatting**: Align fields when beneficial for readability
- **Match expressions**: Consistent formatting for match arms

### Code Block Formatting
```rust
// Good: Multi-line function with proper formatting
fn calculate_complex_result(
    input_parameter: &ComplexType,
    configuration: &Config,
    metadata: Option<&Metadata>,
) -> Result<ProcessedResult, ProcessingError> {
    let intermediate_result = process_input(input_parameter)?;
    let configured_result = apply_configuration(
        intermediate_result,
        configuration,
    )?;
    
    match metadata {
        Some(meta) => enhance_with_metadata(configured_result, meta),
        None => Ok(configured_result),
    }
}

// Good: Struct with consistent field alignment
pub struct MinifierConfig {
    pub optimization_level: OptimizationLevel,
    pub preserve_comments:  bool,
    pub source_maps:        bool,
    pub target_version:     JavaScriptVersion,
}

// Good: Match expression formatting
match processing_result {
    Ok(success_data) => {
        log::info!("Processing completed successfully");
        success_data
    }
    Err(ProcessingError::InvalidInput(msg)) => {
        log::error!("Invalid input: {}", msg);
        return Err(MinifierError::InputValidation(msg));
    }
    Err(other_error) => {
        log::error!("Unexpected processing error: {}", other_error);
        return Err(MinifierError::ProcessingFailed(other_error.to_string()));
    }
}
```

## Style Consistency Rules

### Function Formatting
- **Parameters**: Break long parameter lists across multiple lines
- **Return types**: Place on same line unless it exceeds line limit
- **Body**: Consistent indentation and spacing

### Type Formatting
- **Generics**: Use meaningful type parameter names
- **Bounds**: Format trait bounds consistently
- **Lifetimes**: Minimize explicit lifetime annotations

### Expression Formatting
- **Chain calls**: Break method chains at appropriate points
- **Operators**: Consistent spacing around operators
- **Parentheses**: Use when necessary for clarity

## Pre-commit Formatting Checks

### Required Checks
1. **rustfmt**: Code must be formatted with `cargo fmt`
2. **No trailing whitespace**: Remove all trailing spaces
3. **Consistent line endings**: Use Unix line endings
4. **File endings**: Ensure files end with newline

### CI/CD Integration
```bash
# Format check command
cargo fmt -- --check

# Format code command
cargo fmt

# Integration with CI
- name: Check formatting
  run: cargo fmt -- --check
```

## Editor Configuration

### VS Code Settings
```json
{
    "rust-analyzer.rustfmt.overrideCommand": ["rustfmt", "--edition", "2021"],
    "editor.formatOnSave": true,
    "editor.tabSize": 4,
    "editor.insertSpaces": true,
    "files.trimTrailingWhitespace": true
}
```

### Vim Configuration
```vim
" Rust formatting settings
autocmd FileType rust setlocal tabstop=4 shiftwidth=4 expandtab
autocmd BufWritePre *.rs RustFmt
```

## Common Formatting Patterns

### Error Type Formatting
```rust
#[derive(Debug, thiserror::Error)]
pub enum MinifierError {
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("Invalid configuration: {field} = {value}")]
    InvalidConfiguration { field: String, value: String },
    
    #[error("IO error")]
    IoError(#[from] std::io::Error),
}
```

### Configuration Struct Formatting
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilerConfig {
    /// Input file path
    pub input:              PathBuf,
    /// Output file path (optional)
    pub output:             Option<PathBuf>,
    /// Optimization level
    pub optimization_level: OptimizationLevel,
    /// Enable verbose output
    pub verbose:            bool,
    /// Source map generation
    pub source_maps:        bool,
}
```

### Function Chain Formatting
```rust
let result = input_data
    .iter()
    .filter(|item| item.is_valid())
    .map(|item| item.process())
    .filter_map(|result| result.ok())
    .collect::<Vec<_>>();
```