# Google Coding Standards Parameters for Rust

This document outlines the key parameters and guidelines to meet our coding standards specifically for Rust development
## General Principles

### Code Readability
- **Clarity over cleverness**: Write code that is easy to understand rather than showing off advanced Rust features
- **Self-documenting code**: Use descriptive names that make the code's purpose clear
- **Consistent style**: Maintain uniform formatting using `rustfmt`
- **Idiomatic Rust**: Follow Rust conventions and leverage the type system effectively

### Documentation Requirements
- **Crate-level documentation**: Include comprehensive `//!` comments at crate root
- **Public API documentation**: Document all public functions, structs, enums, and traits with `///`
- **Examples in docs**: Include runnable examples in documentation comments
- **README files**: Provide clear project setup and usage instructions

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

### Import Organization
- **Standard library first**: `std` and `core` imports
- **External crates**: Third-party dependencies
- **Local modules**: Current crate modules
- **Group separation**: Blank line between groups
- **Alphabetical order**: Within each group

```rust
use std::collections::HashMap;
use std::fs::File;

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::config::Config;
use crate::error::AppError;
```

## Naming Conventions

### Variables and Functions
- **snake_case**: All variables, functions, and methods
- **Descriptive names**: Avoid abbreviations, use full words
- **Verb phrases**: Functions should start with verbs (`get_user`, `calculate_total`)

### Types and Traits
- **PascalCase**: Structs, enums, traits, and type aliases
- **Meaningful names**: Types should clearly indicate their purpose
- **Trait naming**: Use adjectives ending in `-able` or `-ing` when appropriate

### Constants and Statics
- **SCREAMING_SNAKE_CASE**: All constants and static variables
- **Descriptive prefixes**: Group related constants with common prefixes

### Modules and Crates
- **snake_case**: Module and crate names
- **Short names**: Prefer concise but clear names
- **Hierarchical**: Use nested modules for logical organization

```rust
// Good naming examples
const MAX_RETRY_ATTEMPTS: u32 = 3;
static GLOBAL_CONFIG: Mutex<Config> = Mutex::new(Config::default());

struct UserAccount {
    user_id: u64,
    account_balance: Decimal,
}

trait Serializable {
    fn serialize(&self) -> Result<Vec<u8>, SerializationError>;
}

fn calculate_monthly_interest(principal: Decimal, rate: f64) -> Decimal {
    // Implementation
}
```

## Code Quality Parameters

### Error Handling
- **Result types**: Use `Result<T, E>` for fallible operations
- **Custom error types**: Define specific error enums for your domain
- **Error propagation**: Use `?` operator for clean error handling
- **Avoid panics**: Only panic on programmer errors, not user errors

```rust
#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Query execution failed")]
    QueryFailed,
    #[error("Transaction rolled back")]
    TransactionRollback,
}

pub fn fetch_user(id: u64) -> Result<User, DatabaseError> {
    let connection = establish_connection()?;
    let user = connection.query_user(id)?;
    Ok(user)
}
```

### Type Safety
- **Strong typing**: Avoid primitive obsession, use newtype pattern
- **Enum variants**: Use enums to represent state and choices
- **Option over null**: Use `Option<T>` instead of nullable pointers
- **Trait bounds**: Use appropriate trait bounds for generic functions

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UserId(u64);

#[derive(Debug)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected { session_id: String },
    Error { reason: String },
}
```

### Memory Management
- **Ownership clarity**: Be explicit about ownership transfer
- **Borrowing rules**: Minimize lifetime annotations where possible
- **Clone judiciously**: Avoid unnecessary cloning, prefer borrowing
- **Smart pointers**: Use `Rc`, `Arc`, `Box` appropriately

## Testing Parameters

### Test Organization
- **Unit tests**: In same file using `#[cfg(test)]` modules
- **Integration tests**: In `tests/` directory
- **Documentation tests**: Include examples that compile and run
- **Benchmark tests**: Use criterion.rs for performance testing

### Test Quality Standards
- **Descriptive test names**: Use `should_` prefix and describe expected behavior
- **AAA pattern**: Arrange, Act, Assert structure
- **Test coverage**: Aim for 80%+ line coverage using `cargo tarpaulin`
- **Property-based testing**: Use `proptest` for complex invariants

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_calculate_interest_correctly_for_positive_principal() {
        // Arrange
        let principal = Decimal::from(1000);
        let rate = 0.05;

        // Act
        let interest = calculate_monthly_interest(principal, rate);

        // Assert
        assert_eq!(interest, Decimal::from_str("4.17").unwrap());
    }

    #[test]
    fn should_return_error_when_database_connection_fails() {
        // Test error conditions
    }
}
```

## Documentation Parameters

### API Documentation
- **Comprehensive docs**: Document all public APIs with clear descriptions
- **Parameter documentation**: Explain each parameter and return value
- **Example usage**: Include practical examples in doc comments
- **Safety notes**: Document unsafe code and invariants

```rust
/// Calculates the monthly interest for a given principal and annual rate.
///
/// This function computes compound interest on a monthly basis using the formula:
/// `monthly_interest = principal * (rate / 12)`
///
/// # Arguments
///
/// * `principal` - The initial amount of money
/// * `rate` - The annual interest rate as a decimal (e.g., 0.05 for 5%)
///
/// # Returns
///
/// Returns the monthly interest amount as a `Decimal`.
///
/// # Examples
///
/// ```
/// use rust_crate::calculate_monthly_interest;
/// use rust_decimal::Decimal;
///
/// let interest = calculate_monthly_interest(Decimal::from(1000), 0.05);
/// assert_eq!(interest, Decimal::from_str("4.17").unwrap());
/// ```
pub fn calculate_monthly_interest(principal: Decimal, rate: f64) -> Decimal {
    // Implementation
}
```

### Internal Documentation
- **Complex algorithms**: Explain non-obvious logic with comments
- **Safety invariants**: Document unsafe code requirements
- **Performance notes**: Explain performance characteristics when relevant
- **TODO comments**: Include tracking issue numbers

## Security Parameters

### Safe Rust Practices
- **Avoid unsafe**: Use unsafe code only when absolutely necessary
- **Input validation**: Validate all external inputs at boundaries
- **Secure defaults**: Choose secure options by default
- **Constant-time operations**: Use constant-time algorithms for cryptographic operations

### Dependency Management
- **Audit dependencies**: Regularly run `cargo audit`
- **Minimal dependencies**: Only include necessary dependencies
- **Version pinning**: Pin versions in production deployments
- **Security reviews**: Review security-critical dependencies

## Performance Parameters

### Efficiency Guidelines
- **Profile first**: Use profiling tools before optimizing
- **Appropriate collections**: Choose correct data structures for use case
- **Lazy evaluation**: Use iterators instead of collecting when possible
- **Avoid allocations**: Minimize heap allocations in hot paths

```rust
// Prefer iterators over collecting
fn process_items(items: &[Item]) -> impl Iterator<Item = ProcessedItem> + '_ {
    items.iter()
        .filter(|item| item.is_active())
        .map(|item| item.process())
}

// Use appropriate collection types
use std::collections::{HashMap, BTreeMap, HashSet};
use indexmap::IndexMap; // When insertion order matters
```

### Async/Concurrency
- **Tokio runtime**: Use tokio for async applications
- **Structured concurrency**: Prefer scoped threads and proper cleanup
- **Channel usage**: Use appropriate channel types for communication
- **Blocking operations**: Move blocking operations to dedicated thread pools

## Build and Deployment Parameters

### Cargo Configuration
- **Edition**: Use latest stable Rust edition
- **Features**: Use feature flags for optional functionality
- **Workspace**: Organize large projects using cargo workspaces
- **Profiles**: Configure appropriate build profiles

```toml
[package]
name = "my-rust-app"
version = "0.1.0"
edition = "2021"
rust-version = "1.70"

[features]
default = ["json-support"]
json-support = ["serde_json"]
database = ["sqlx", "tokio"]

[profile.release]
lto = true
codegen-units = 1
panic = "abort"

[profile.dev]
debug = true
```

### CI/CD Requirements
- **Automated testing**: All tests must pass
- **Clippy lints**: Code must pass `cargo clippy`
- **Format check**: Code must be formatted with `rustfmt`
- **Security audit**: Run `cargo audit` in CI pipeline
- **Documentation**: Generate and check documentation with `cargo doc`

### Tooling Integration
- **IDE setup**: Configure rust-analyzer for better development experience
- **Pre-commit hooks**: Run formatting and linting before commits
- **Dependency updates**: Regularly update dependencies with `cargo update`
- **Cross-compilation**: Test on target platforms when applicable