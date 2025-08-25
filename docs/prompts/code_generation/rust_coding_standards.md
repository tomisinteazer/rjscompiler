# Rust Code Generation Standards

## General Code Generation Principles

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

## Code Quality Standards

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

## Import Organization
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

## API Documentation Standards

### Comprehensive Documentation
- **Function documentation**: Document all public APIs with clear descriptions
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

## Performance Guidelines

### Efficiency Standards
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

## Security Standards

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