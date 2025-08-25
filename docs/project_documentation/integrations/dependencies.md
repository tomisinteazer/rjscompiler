# Dependencies Documentation

## Overview

This document outlines all external crates and integrations used in the JavaScript minifier project, including their purpose, version requirements, and integration details.

## Core Dependencies

### Command Line Interface
#### clap (v4.5+)
- **Purpose**: Command-line argument parsing and help generation
- **Features**: Derive macros for easy CLI definition
- **Integration**: Main CLI interface in `src/main.rs`
- **Documentation**: https://docs.rs/clap

```toml
clap = { version = "4.5", features = ["derive"] }
```

### Error Handling
#### thiserror (v1.0+)
- **Purpose**: Ergonomic error type definitions
- **Features**: Derive macros for error types
- **Integration**: All error types throughout the project
- **Documentation**: https://docs.rs/thiserror

```toml
thiserror = "1.0"
```

## Planned Dependencies

### JavaScript Parsing
#### swc_ecma_parser (v0.140+)
- **Purpose**: High-performance JavaScript/TypeScript parser
- **Features**: ES2022+ support, error recovery, source maps
- **Integration**: Core parsing engine in `parser` module
- **Alternative**: Custom parser implementation
- **Documentation**: https://docs.rs/swc_ecma_parser

```toml
swc_ecma_parser = "0.140"
swc_ecma_ast = "0.110"
swc_common = "0.33"
```

### AST Manipulation
#### swc_ecma_transforms (v0.225+)
- **Purpose**: AST transformation utilities
- **Features**: Visitor patterns, transformation passes
- **Integration**: Transformer component for code optimization
- **Documentation**: https://docs.rs/swc_ecma_transforms

```toml
swc_ecma_transforms = "0.225"
swc_ecma_transforms_base = "0.135"
```

### Code Generation
#### swc_ecma_codegen (v0.146+)
- **Purpose**: AST to JavaScript code generation
- **Features**: Minified output, source maps, formatting options
- **Integration**: Generator component for final output
- **Documentation**: https://docs.rs/swc_ecma_codegen

```toml
swc_ecma_codegen = "0.146"
```

## Utility Dependencies

### File System Operations
#### glob (v0.3+)
- **Purpose**: Pattern matching for file discovery
- **Features**: Recursive directory traversal, pattern matching
- **Integration**: Batch processing functionality
- **Documentation**: https://docs.rs/glob

```toml
glob = "0.3"
```

#### walkdir (v2.4+)
- **Purpose**: Recursive directory walking
- **Features**: Efficient directory traversal, filtering
- **Integration**: File discovery and processing
- **Documentation**: https://docs.rs/walkdir

```toml
walkdir = "2.4"
```

### Serialization
#### serde (v1.0+)
- **Purpose**: Serialization framework
- **Features**: JSON, TOML, YAML support for configuration
- **Integration**: Configuration file parsing
- **Documentation**: https://docs.rs/serde

```toml
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"
serde_yaml = "0.9"
```

### Parallel Processing
#### rayon (v1.8+)
- **Purpose**: Data parallelism library
- **Features**: Parallel iterators, thread pools
- **Integration**: Batch processing and parallel transformations
- **Documentation**: https://docs.rs/rayon

```toml
rayon = "1.8"
```

### Performance Utilities
#### dashmap (v5.5+)
- **Purpose**: Concurrent hash map
- **Features**: Thread-safe, high-performance caching
- **Integration**: Symbol table and scope caching
- **Documentation**: https://docs.rs/dashmap

```toml
dashmap = "5.5"
```

#### ahash (v0.8+)
- **Purpose**: High-performance hash function
- **Features**: Fast, DoS-resistant hashing
- **Integration**: HashMap backends for better performance
- **Documentation**: https://docs.rs/ahash

```toml
ahash = "0.8"
```

## Development Dependencies

### Testing
#### proptest (v1.4+)
- **Purpose**: Property-based testing framework
- **Features**: Random test case generation, shrinking
- **Integration**: Comprehensive testing of transformations
- **Documentation**: https://docs.rs/proptest

```toml
[dev-dependencies]
proptest = "1.4"
```

#### criterion (v0.5+)
- **Purpose**: Benchmarking framework
- **Features**: Statistical analysis, performance regression detection
- **Integration**: Performance testing and optimization validation
- **Documentation**: https://docs.rs/criterion

```toml
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }
```

### Code Quality
#### clippy (built-in)
- **Purpose**: Rust linting tool
- **Features**: Code quality checks, style enforcement
- **Integration**: CI/CD pipeline, development workflow
- **Documentation**: https://doc.rust-lang.org/clippy/

#### rustfmt (built-in)
- **Purpose**: Code formatting tool
- **Features**: Consistent code style, automatic formatting
- **Integration**: Pre-commit hooks, CI/CD validation
- **Documentation**: https://rust-lang.github.io/rustfmt/

## Optional Dependencies

### Source Maps
#### source-map (v0.12+)
- **Purpose**: Source map generation and manipulation
- **Features**: V3 source map format, base64 encoding
- **Integration**: Debug information preservation
- **Documentation**: https://docs.rs/source-map

```toml
source-map = { version = "0.12", optional = true }
```

### Configuration
#### figment (v0.10+)
- **Purpose**: Configuration management framework
- **Features**: Multiple sources, type-safe configuration
- **Integration**: Advanced configuration handling
- **Documentation**: https://docs.rs/figment

```toml
figment = { version = "0.10", optional = true }
```

### Logging
#### tracing (v0.1+)
- **Purpose**: Application-level tracing framework
- **Features**: Structured logging, performance tracing
- **Integration**: Debug and performance analysis
- **Documentation**: https://docs.rs/tracing

```toml
tracing = { version = "0.1", optional = true }
tracing-subscriber = { version = "0.3", optional = true }
```

## Feature Flags

### Default Features
```toml
[features]
default = ["cli", "parallel"]
cli = ["clap"]
parallel = ["rayon"]
source-maps = ["source-map"]
advanced-config = ["figment"]
tracing = ["dep:tracing", "tracing-subscriber"]
```

### Feature Combinations
- **minimal**: No optional features, smallest binary
- **full**: All features enabled for maximum functionality
- **performance**: Focus on speed optimizations
- **debug**: Enhanced debugging and tracing capabilities

## Version Compatibility

### Minimum Supported Rust Version (MSRV)
- **Rust 1.70.0**: Required for latest language features and dependencies
- **Edition 2021**: Use latest Rust edition for best performance

### Dependency Updates
- **Patch versions**: Automatic updates for bug fixes
- **Minor versions**: Manual review for new features
- **Major versions**: Careful evaluation for breaking changes

## Build Configuration

### Cargo.toml Example
```toml
[package]
name = "rjs-compiler"
version = "0.1.0"
edition = "2021"
rust-version = "1.70"

[dependencies]
clap = { version = "4.5", features = ["derive"] }
thiserror = "1.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Parser dependencies (planned)
swc_ecma_parser = { version = "0.140", optional = true }
swc_ecma_ast = { version = "0.110", optional = true }
swc_common = { version = "0.33", optional = true }

# Utility dependencies
glob = "0.3"
rayon = { version = "1.8", optional = true }

[dev-dependencies]
proptest = "1.4"
criterion = { version = "0.5", features = ["html_reports"] }

[features]
default = ["cli"]
cli = ["clap"]
parallel = ["rayon"]
swc-parser = ["swc_ecma_parser", "swc_ecma_ast", "swc_common"]

[[bench]]
name = "minification_benchmark"
harness = false
```

## Integration Guidelines

### Adding New Dependencies
1. **Evaluate necessity**: Ensure dependency is truly needed
2. **Check alternatives**: Consider existing solutions
3. **Review license**: Ensure compatible licensing
4. **Security audit**: Check for known vulnerabilities
5. **Performance impact**: Measure compilation and runtime impact
6. **Documentation**: Update this file with new dependencies

### Dependency Management
- **Regular updates**: Keep dependencies current for security
- **Version pinning**: Pin versions in production builds
- **Audit schedule**: Regular security audits with `cargo audit`
- **Minimal dependencies**: Avoid unnecessary dependencies

---

*Status*: 📋 Planned  
*Owner*: JavaScript Minifier Team  
*Last Updated*: 2025-08-25