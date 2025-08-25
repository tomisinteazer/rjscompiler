# CLI Commands Documentation

## Overview

The RJS Compiler provides a comprehensive command-line interface for JavaScript minification with various options and modes of operation.

## Command Structure

### Basic Syntax
```bash
rjs-compiler [OPTIONS] <INPUT_FILE>
```

### Global Options
- `--version`, `-V`: Show version information
- `--help`, `-h`: Display help information
- `--verbose`, `-v`: Enable detailed output
- `--quiet`, `-q`: Suppress non-error output

## Primary Commands

### Minify Command (Default)
```bash
rjs-compiler input.js
rjs-compiler --output minified.js input.js
rjs-compiler -o minified.js -v input.js
```

**Description**: Minifies a JavaScript file with aggressive optimization.

**Options**:
- `--output`, `-o`: Specify output file (default: stdout)
- `--source-map`: Generate source map file
- `--preserve-comments`: Keep certain comments (license headers)
- `--target`: Target JavaScript version (es5, es6, es2017, etc.)

### Batch Processing
```bash
rjs-compiler --batch src/**/*.js --output-dir dist/
rjs-compiler -b "src/**/*.js" -d dist/ --parallel
```

**Description**: Process multiple files in parallel.

**Options**:
- `--batch`, `-b`: Batch processing mode with glob patterns
- `--output-dir`, `-d`: Output directory for processed files
- `--parallel`: Enable parallel processing
- `--preserve-structure`: Maintain directory structure

### Analysis Mode
```bash
rjs-compiler --analyze input.js
rjs-compiler --analyze --report analysis.json input.js
```

**Description**: Analyze code without minification.

**Options**:
- `--analyze`: Enable analysis-only mode
- `--report`: Generate detailed analysis report
- `--stats`: Show compression statistics
- `--scope-analysis`: Display scope and symbol information

## Advanced Options

### Optimization Levels
```bash
rjs-compiler --level aggressive input.js
rjs-compiler --level safe input.js
rjs-compiler --level custom --config config.json input.js
```

**Levels**:
- `safe`: Conservative optimizations (default)
- `aggressive`: Maximum compression with some risk
- `custom`: User-defined optimization rules

### Configuration
```bash
rjs-compiler --config minify.config.json input.js
rjs-compiler --preset production input.js
rjs-compiler --preset development input.js
```

**Options**:
- `--config`: Load configuration from JSON file
- `--preset`: Use predefined configuration preset
- `--no-rename`: Disable variable renaming
- `--no-mangle`: Disable function name mangling

### Output Control
```bash
rjs-compiler --format compact input.js
rjs-compiler --format readable input.js
rjs-compiler --inline-source-map input.js
```

**Formats**:
- `compact`: Single line, minimal size (default)
- `readable`: Some formatting for debugging
- `pretty`: Human-readable output

## Examples

### Basic Minification
```bash
# Minify to stdout
rjs-compiler app.js

# Minify to file
rjs-compiler -o app.min.js app.js

# Verbose minification
rjs-compiler -v -o app.min.js app.js
```

### Advanced Usage
```bash
# Aggressive minification with source maps
rjs-compiler --level aggressive --source-map -o app.min.js app.js

# Batch processing with parallel execution
rjs-compiler -b "src/**/*.js" -d dist/ --parallel --preserve-structure

# Analysis without minification
rjs-compiler --analyze --report analysis.json --stats app.js
```

### Configuration-Based
```bash
# Use configuration file
rjs-compiler --config production.json app.js

# Production preset
rjs-compiler --preset production -o app.min.js app.js

# Development preset (less aggressive)
rjs-compiler --preset development app.js
```

## Configuration File Format

### Example Configuration (minify.config.json)
```json
{
    "level": "aggressive",
    "target": "es6",
    "sourceMap": true,
    "preserveComments": ["@license"],
    "renaming": {
        "variables": true,
        "functions": true,
        "properties": false
    },
    "optimization": {
        "deadCodeElimination": true,
        "constantFolding": true,
        "functionInlining": false
    },
    "output": {
        "format": "compact",
        "quotes": "single",
        "semicolons": "minimal"
    }
}
```

### Configuration Options
- **level**: Optimization aggressiveness
- **target**: JavaScript version compatibility
- **sourceMap**: Enable source map generation
- **preserveComments**: Comments to preserve
- **renaming**: Control identifier renaming
- **optimization**: Enable/disable specific optimizations
- **output**: Control output formatting

## Exit Codes

- `0`: Success
- `1`: General error
- `2`: Invalid arguments
- `3`: File not found
- `4`: Parsing error
- `5`: Minification error
- `6`: Output error

## Environment Variables

### Configuration
- `RJS_CONFIG`: Default configuration file path
- `RJS_PRESET`: Default preset to use
- `RJS_PARALLEL`: Enable parallel processing by default
- `RJS_VERBOSE`: Enable verbose output by default

### Performance
- `RJS_MAX_THREADS`: Maximum number of parallel threads
- `RJS_MEMORY_LIMIT`: Memory usage limit for large files
- `RJS_CACHE_DIR`: Directory for caching analysis results

## Error Handling

### Common Errors
- **Syntax errors**: Invalid JavaScript input
- **Permission errors**: Cannot read input or write output
- **Memory errors**: File too large for available memory
- **Configuration errors**: Invalid configuration options

### Error Output Format
```
Error: Syntax error in input file
  --> app.js:15:23
   |
15 | function test( {
   |                ^
   | Expected closing parenthesis
```

## Performance Tips

### Optimization
- Use `--parallel` for multiple files
- Enable caching with `RJS_CACHE_DIR`
- Use appropriate optimization level
- Consider target environment compatibility

### Memory Usage
- Set `RJS_MEMORY_LIMIT` for large files
- Use streaming mode for very large inputs
- Monitor memory usage with `--verbose`

## Integration

### Build Tools
```bash
# Webpack integration
webpack --plugin rjs-compiler-webpack-plugin

# Gulp integration  
gulp.task('minify', () => {
    return gulp.src('src/**/*.js')
        .pipe(rjsCompiler({ level: 'aggressive' }))
        .pipe(gulp.dest('dist/'));
});

# NPM scripts
{
    "scripts": {
        "minify": "rjs-compiler -b 'src/**/*.js' -d dist/",
        "build": "rjs-compiler --preset production -o bundle.min.js bundle.js"
    }
}
```

---

*Status*: ✅ Implemented  
*Owner*: JavaScript Minifier Team  
*Last Updated*: 2025-08-25