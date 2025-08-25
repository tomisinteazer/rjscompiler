# RJS Compiler - Usage Guide

Welcome to the RJS Compiler usage documentation! This guide will help you understand how to use the RJS Compiler, a high-performance JavaScript compiler built with Rust.

## Table of Contents

- [Overview](#overview)
- [Installation](#installation)
- [Basic Usage](#basic-usage)
- [Command-Line Options](#command-line-options)
- [Examples](#examples)
- [Error Handling](#error-handling)
- [Verbose Mode](#verbose-mode)
- [Troubleshooting](#troubleshooting)
- [Support](#support)

## Overview

RJS Compiler is a Rust-based JavaScript compiler that provides fast and reliable JavaScript compilation. The CLI tool processes JavaScript source files and transforms them according to specified compilation rules and optimizations.

### Key Features

- 🚀 **Fast compilation** using Rust's performance characteristics
- 🔍 **Verbose output** for debugging compilation processes
- 🛡️ **Comprehensive error handling** and reporting
- 🌐 **Cross-platform compatibility** (Windows, macOS, Linux)
- 📚 **Detailed help system** with comprehensive documentation

## Installation

### Prerequisites

- Rust 1.70 or later
- Cargo (included with Rust)

### Building from Source

1. Clone the repository:
   ```bash
   git clone <your-repo-url>
   cd rjscompiler
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

3. The compiled binary will be available at `target/release/rjs-compiler`

### Running without Installation

You can run the compiler directly using Cargo:
```bash
cargo run -- [OPTIONS] <FILE>
```

## Basic Usage

The basic syntax for using RJS Compiler is:

```bash
rjs-compiler [OPTIONS] <FILE>
```

### Minimum Example

To compile a JavaScript file:
```bash
rjs-compiler my_script.js
```

This will:
- Display a welcome message
- Process the specified JavaScript file
- Show compilation success status

## Command-Line Options

### Arguments

| Argument | Description | Required |
|----------|-------------|----------|
| `<FILE>` | Path to the JavaScript source file to compile | Yes* |

*Required unless you want to see usage information

### Flags

| Flag | Short | Long | Description |
|------|-------|------|-------------|
| `-v` | `-v` | `--verbose` | Enable verbose output mode with detailed compilation information |
| `-h` | `-h` | `--help` | Display help information and usage instructions |
| `-V` | `-V` | `--version` | Show the version of RJS Compiler |

### Getting Help

For detailed help information:
```bash
rjs-compiler --help
```

For version information:
```bash
rjs-compiler --version
```

## Examples

### Basic Compilation

Compile a simple JavaScript file:
```bash
rjs-compiler example.js
```

**Output:**
```
🦀 Hello Rust!
Welcome to RJS Compiler v0.1.0
✅ Compilation completed successfully!
```

### Verbose Compilation

Get detailed information about the compilation process:
```bash
rjs-compiler --verbose example.js
```

**Output:**
```
🦀 Hello Rust!
Welcome to RJS Compiler v0.1.0
🔍 Verbose mode enabled
📋 Configuration:
   📁 Input file: example.js
   🔧 Verbose output: true
📂 Processing input file: example.js
✅ Input file validation passed
🚀 Starting compilation process...
🔄 Phase 1: Lexical analysis
🔄 Phase 2: Syntax parsing
🔄 Phase 3: Semantic analysis
🔄 Phase 4: Code generation
🔄 Phase 5: Optimization
✅ Compilation completed successfully!
📊 Compilation statistics:
   ⏱️  Duration: <measurement pending>
   📏 Output size: <measurement pending>
```

### Using Short Flags

You can use the short form of flags:
```bash
rjs-compiler -v my_complex_app.js
```

### Running from Different Directories

You can specify files with full or relative paths:
```bash
# Relative path
rjs-compiler ./src/main.js

# Absolute path
rjs-compiler /home/user/projects/app/script.js
```

## Error Handling

RJS Compiler provides clear error messages for common issues:

### File Not Found

```bash
rjs-compiler nonexistent.js
```

**Output:**
```
🦀 Hello Rust!
Welcome to RJS Compiler v0.1.0
Error: File not found: nonexistent.js
```

### No Input File Specified

```bash
rjs-compiler
```

**Output:**
```
🦀 Hello Rust!
Welcome to RJS Compiler v0.1.0
💡 Usage: rjs-compiler [OPTIONS] <FILE>
   Use --help for more information
   Example: rjs-compiler --verbose my_script.js
Error: Input file not specified
```

### Exit Codes

- `0`: Successful compilation
- `1`: Error occurred (file not found, compilation failed, etc.)

## Verbose Mode

The verbose mode (`--verbose` or `-v`) provides detailed insights into the compilation process:

### What Verbose Mode Shows

1. **Configuration Display**: Shows the current compiler settings
2. **File Processing**: Indicates which file is being processed
3. **Validation Status**: Confirms input file validation
4. **Compilation Phases**: Shows each phase of compilation:
   - Lexical analysis
   - Syntax parsing
   - Semantic analysis
   - Code generation
   - Optimization
5. **Statistics**: Compilation metrics (planned feature)

### When to Use Verbose Mode

- **Debugging**: When you need to understand what the compiler is doing
- **Performance Analysis**: To see which phases take the most time
- **Learning**: To understand the compilation process
- **Troubleshooting**: When compilation fails and you need details

## Troubleshooting

### Common Issues and Solutions

#### Issue: "File not found" error
**Solution:** 
- Check that the file path is correct
- Verify the file exists in the specified location
- Ensure you have read permissions for the file

#### Issue: Command not found
**Solution:**
- Make sure you've built the project with `cargo build --release`
- If using `cargo run`, ensure you're in the project directory
- Check that Rust and Cargo are properly installed

#### Issue: Permission denied
**Solution:**
- Ensure you have read permissions for the input file
- On Unix systems, you might need to adjust file permissions with `chmod`

### Getting More Information

1. Use `--verbose` flag for detailed output
2. Check the file exists: `ls -la your_file.js`
3. Verify file permissions: `ls -l your_file.js`

## Support

### Documentation

- **Project README**: See [README.md](README.md) for project overview
- **Code Documentation**: Run `cargo doc --open` for API documentation
- **Examples**: Check the [example.js](example.js) file in the project root

### Reporting Issues

When reporting issues, please include:

1. **Command used**: The exact command that caused the issue
2. **Expected behavior**: What you expected to happen
3. **Actual behavior**: What actually happened
4. **Error messages**: Any error messages displayed
5. **Environment**: Operating system and Rust version
6. **File information**: Details about the JavaScript file being compiled

### Development Status

This is version 0.1.0 of RJS Compiler. Current status:

- ✅ **CLI Interface**: Fully functional
- ✅ **Argument Parsing**: Complete
- ✅ **Error Handling**: Implemented
- ✅ **Verbose Mode**: Available
- 🚧 **JavaScript Parsing**: In development
- 🚧 **Compilation Logic**: Planned
- 🚧 **Optimization**: Planned

## Advanced Usage

### Environment Variables

Currently, RJS Compiler doesn't use environment variables, but this may change in future versions.

### Configuration Files

Configuration file support is planned for future releases.

### Output Options

Output configuration options will be added in future versions.

## Contributing

For information about contributing to RJS Compiler, please see the project documentation in the `docs/` folder.

### Development Workflow

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `cargo test`
5. Run linting: `cargo clippy`
6. Format code: `cargo fmt`
7. Submit a pull request

### Code Standards

This project follows Google Coding Standards for Rust. See the documentation in `docs/prompts/code_generation/Standards.md` for detailed guidelines.

## FAQ

### Q: What JavaScript features are supported?
**A:** This is version 0.1.0, and JavaScript parsing is still in development. Full feature support will be documented as it becomes available.

### Q: Can I compile multiple files at once?
**A:** Currently, the compiler processes one file at a time. Batch processing may be added in future versions.

### Q: Is there a GUI version?
**A:** Currently, only the CLI version is available. A GUI may be considered for future releases.

### Q: What output formats are supported?
**A:** Output format options are being developed and will be documented when available.

---

**Version**: 0.1.0  
**Last Updated**: 2025-08-25  
**Rust Edition**: 2021  
**License**: TBD