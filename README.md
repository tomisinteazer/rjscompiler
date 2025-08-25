# JavaScript Minifier in Rust (RJS Compiler)

🦀 **High-Performance JavaScript Minifier** - An aggressive JavaScript minification tool built with Rust

## Project Goal

Build a high-performance JavaScript minifier written in Rust that **aggressively reduces code size while preserving 100% functionality**. This tool targets complete, isolated JavaScript components and applies extreme compression techniques to dramatically reduce file sizes for faster web applications.

## Core Objectives

### 🎯 **Extreme Size Reduction**
- **Variable renaming**: `const userPreferences` → `const a`
- **Function renaming**: `function calculateScore()` → `function b()`
- **Property minification**: `config.animationDuration` → `config.a` (when safe)
- **Maximum compression**: Remove all unnecessary characters, whitespace, and formatting
- **Target reduction**: 70-90% smaller file sizes

### 🔒 **Functionality Preservation**
- Maintain exact JavaScript semantics
- Preserve all runtime behavior
- Handle scoping, hoisting, and `this` binding correctly
- Ensure minified code executes identically to original
- Zero behavioral changes guarantee

### 📦 **Self-Contained Components**
- Target complete, isolated JavaScript components
- Handle both UI structure and functionality in one file
- Optimize entire component as a unit
- Support for modern module systems

## Key Technical Challenges

### ⚡ **Safe Aggressive Minification**
- **Scope analysis** to avoid naming collisions
- **Reference tracking** to preserve variable relationships
- **Dead code elimination** without breaking functionality
- **Property access safety** (dot notation vs bracket notation)
- **Symbol table construction** for safe renaming

### 🚀 **JavaScript Language Compliance**
- Handle all **ES6+ features** (arrow functions, destructuring, modules)
- Proper parsing of **edge cases** (regex literals, template strings)
- Maintain **execution order** and side effects
- Support for **modern syntax** and language constructs
- **AST-based processing** for accuracy

## Implementation Approach

### 🔧 **Multi-Stage Pipeline**
1. **🔍 Parse**: Convert JavaScript to Abstract Syntax Tree (AST)
2. **📊 Analyze**: Build scope tree and symbol table
3. **⚙️ Transform**: Apply aggressive minification passes
4. **📤 Generate**: Output highly compressed JavaScript

### 🦀 **Rust Advantages**
- **⚡ Blazing fast performance** for processing large codebases
- **🛡️ Memory safety** without garbage collection overhead
- **🔧 Excellent tooling** and package ecosystem
- **🏗️ Perfect for compiler development** and language tooling
- **🚀 Zero-cost abstractions** for maximum performance

## Current Status

### ✅ **Completed**
- CLI framework with clap integration
- Comprehensive error handling with thiserror
- Verbose output and debugging modes
- Professional documentation structure
- File input validation and processing pipeline

### 🚧 **In Development**
- JavaScript lexical analysis and parsing
- AST construction and manipulation
- Scope analysis and symbol table generation
- Minification transformation passes

### 📋 **Planned Features**
- Variable and function name mangling
- Dead code elimination
- Property access optimization
- Output size reporting and statistics
- Configuration file support
- Batch processing capabilities

## Installation

Make sure you have Rust 1.70+ installed, then clone and build:

```bash
git clone <your-repo-url>
cd rjscompiler
cargo build --release
```

## Usage

### Basic minification
```bash
cargo run -- input.js
```

### Enable verbose output for detailed processing information
```bash
cargo run -- --verbose input.js
```

### Get comprehensive help
```bash
cargo run -- --help
```

For detailed usage instructions, see [USAGE.md](USAGE.md).

## Expected Outcomes

- **📉 Dramatically reduced file sizes** (often 70-90% smaller)
- **⚡ Faster network transfer** and parsing times
- **🔒 Maintained functionality** with zero behavioral changes
- **🏭 Production-ready** minification for web applications
- **🚀 Superior performance** compared to existing minifiers

## Development

This project uses:
- **Rust 2024 edition** for latest language features
- **clap 4.0** for command-line argument parsing
- **thiserror 1.0** for robust error handling
- **Professional documentation** following Google Rust standards

### Project Structure
```
rjscompiler/
├── src/
│   └── main.rs          # Main CLI application
├── docs/                # Comprehensive documentation
│   ├── prompts/         # LLM prompts for development
│   ├── project_documentation/ # Technical specs
│   ├── work_tracking/   # Progress tracking
│   ├── resources/       # Development resources
│   └── templates/       # Code templates
├── Cargo.toml           # Project configuration
├── README.md           # This file
├── USAGE.md            # Detailed usage guide
└── example.js          # Test JavaScript file
```

## Technical Vision

This project combines **compiler theory**, **language parsing**, and **optimization techniques** to create a tool that makes web applications smaller and faster while maintaining complete compatibility. By leveraging Rust's performance characteristics and safety guarantees, we aim to build the fastest and most reliable JavaScript minifier available.

## Contributing

We welcome contributions! Please see the documentation in the `docs/` folder for:
- Code style guidelines (`docs/prompts/code_generation/Standards.md`)
- Development workflow
- Architecture decisions
- Testing standards

## License

TBD