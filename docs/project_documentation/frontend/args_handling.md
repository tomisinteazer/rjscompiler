# Argument Handling Documentation

## Overview

The argument handling system processes command-line inputs, validates options, and configures the minification pipeline based on user preferences.

## Architecture

### Argument Parser (clap-based)
- **Command definition**: Define all available commands and options
- **Type validation**: Ensure arguments match expected types
- **Conflict resolution**: Handle mutually exclusive options
- **Help generation**: Automatic help text generation

### Configuration Builder
- **Option merging**: Combine CLI args, config files, and defaults
- **Validation**: Ensure configuration is valid and consistent
- **Environment variables**: Support for environment-based configuration
- **Preset handling**: Load and apply predefined configurations

## Implementation Details

### Clap Configuration
```rust
use clap::{Arg, Command, ArgMatches};

fn build_cli() -> Command {
    Command::new("rjs-compiler")
        .version(env!("CARGO_PKG_VERSION"))
        .about("High-performance JavaScript minifier")
        .arg(
            Arg::new("input")
                .help("Input JavaScript file")
                .required(true)
                .value_parser(clap::value_parser!(PathBuf))
                .index(1),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .help("Output file path")
                .value_parser(clap::value_parser!(PathBuf)),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(clap::ArgAction::SetTrue)
                .help("Enable verbose output"),
        )
        // ... more arguments
}
```

### Argument Processing Pipeline
1. **Parse CLI arguments**: Extract and validate command-line options
2. **Load configuration**: Read config files if specified
3. **Merge settings**: Combine CLI, config, and environment variables
4. **Validate configuration**: Ensure all settings are valid
5. **Build processor config**: Create minification configuration

## Configuration Sources

### Priority Order (highest to lowest)
1. **Command-line arguments**: Direct CLI options
2. **Configuration files**: JSON/TOML configuration files
3. **Environment variables**: System environment settings
4. **Preset configurations**: Predefined configuration templates
5. **Default values**: Built-in default settings

### Configuration Merging Strategy
```rust
#[derive(Debug, Clone)]
pub struct MinifierConfig {
    pub input: PathBuf,
    pub output: Option<PathBuf>,
    pub optimization_level: OptimizationLevel,
    pub source_map: bool,
    pub target: JavaScriptTarget,
    pub preserve_comments: Vec<String>,
    pub verbose: bool,
}

impl MinifierConfig {
    pub fn from_args(matches: &ArgMatches) -> Result<Self, ConfigError> {
        let mut config = Self::default();
        
        // Apply CLI arguments
        config.input = matches.get_one::<PathBuf>("input")
            .ok_or(ConfigError::MissingInput)?
            .clone();
            
        if let Some(output) = matches.get_one::<PathBuf>("output") {
            config.output = Some(output.clone());
        }
        
        config.verbose = matches.get_flag("verbose");
        
        // Load and merge configuration file if specified
        if let Some(config_path) = matches.get_one::<PathBuf>("config") {
            let file_config = Self::from_file(config_path)?;
            config = config.merge(file_config)?;
        }
        
        // Apply environment variables
        config = config.merge_env()?;
        
        // Validate final configuration
        config.validate()?;
        
        Ok(config)
    }
}
```

## Validation Rules

### Input Validation
- **File existence**: Ensure input files exist and are readable
- **File extensions**: Validate JavaScript file extensions (.js, .mjs, .jsx)
- **File size limits**: Check against maximum file size limits
- **Permissions**: Verify read permissions on input files

### Output Validation
- **Directory existence**: Ensure output directories exist or can be created
- **Write permissions**: Verify write access to output locations
- **Filename conflicts**: Handle existing file overwrite scenarios
- **Path resolution**: Resolve relative paths correctly

### Option Validation
- **Mutually exclusive**: Handle conflicting options gracefully
- **Dependency requirements**: Ensure dependent options are provided
- **Value ranges**: Validate numeric options are within acceptable ranges
- **Enum validation**: Ensure string options match predefined values

## Error Handling

### Argument Errors
```rust
#[derive(Debug, thiserror::Error)]
pub enum ArgumentError {
    #[error("Input file not found: {0}")]
    InputNotFound(PathBuf),
    
    #[error("Invalid optimization level: {0}")]
    InvalidOptimizationLevel(String),
    
    #[error("Conflicting options: {0} and {1}")]
    ConflictingOptions(String, String),
    
    #[error("Missing required argument: {0}")]
    MissingArgument(String),
    
    #[error("Invalid file format: {0}")]
    InvalidFileFormat(String),
}
```

### Error Recovery
- **Graceful degradation**: Use safe defaults when possible
- **User guidance**: Provide helpful error messages with suggestions
- **Exit codes**: Return appropriate exit codes for different error types
- **Partial processing**: Continue processing valid files in batch mode

## Advanced Features

### Glob Pattern Support
```rust
use glob::glob;

fn expand_glob_patterns(patterns: &[String]) -> Result<Vec<PathBuf>, GlobError> {
    let mut files = Vec::new();
    
    for pattern in patterns {
        for entry in glob(pattern)? {
            match entry {
                Ok(path) => {
                    if path.extension() == Some(OsStr::new("js")) {
                        files.push(path);
                    }
                }
                Err(e) => eprintln!("Warning: {}", e),
            }
        }
    }
    
    Ok(files)
}
```

### Configuration File Loading
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ConfigFile {
    pub level: Option<String>,
    pub target: Option<String>,
    pub source_map: Option<bool>,
    pub preserve_comments: Option<Vec<String>>,
    pub renaming: Option<RenamingConfig>,
    pub optimization: Option<OptimizationConfig>,
}

impl ConfigFile {
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)?;
        
        match path.extension().and_then(|s| s.to_str()) {
            Some("json") => Ok(serde_json::from_str(&content)?),
            Some("toml") => Ok(toml::from_str(&content)?),
            Some("yaml") | Some("yml") => Ok(serde_yaml::from_str(&content)?),
            _ => Err(ConfigError::UnsupportedFormat(path.to_path_buf())),
        }
    }
}
```

### Environment Variable Support
```rust
use std::env;

impl MinifierConfig {
    fn merge_env(mut self) -> Result<Self, ConfigError> {
        if let Ok(level) = env::var("RJS_LEVEL") {
            self.optimization_level = level.parse()?;
        }
        
        if let Ok(target) = env::var("RJS_TARGET") {
            self.target = target.parse()?;
        }
        
        if env::var("RJS_VERBOSE").is_ok() {
            self.verbose = true;
        }
        
        if let Ok(threads) = env::var("RJS_MAX_THREADS") {
            self.max_threads = Some(threads.parse()?);
        }
        
        Ok(self)
    }
}
```

## Performance Considerations

### Argument Parsing Optimization
- **Lazy evaluation**: Parse arguments only when needed
- **Caching**: Cache parsed configurations for repeated use
- **Minimal allocations**: Avoid unnecessary string allocations
- **Fast validation**: Optimize validation for common cases

### Memory Management
- **Configuration size**: Keep configuration structures small
- **String interning**: Intern commonly used strings
- **Clone optimization**: Minimize expensive cloning operations
- **Reference counting**: Use Rc/Arc for shared configurations

## Testing Strategy

### Unit Tests
- **Individual argument parsing**: Test each argument type
- **Validation logic**: Test all validation rules
- **Error conditions**: Test error handling scenarios
- **Edge cases**: Test boundary conditions and corner cases

### Integration Tests
- **Full CLI workflows**: Test complete argument processing
- **Configuration merging**: Test multiple configuration sources
- **File handling**: Test file input/output scenarios
- **Environment interaction**: Test environment variable handling

### Property-Based Testing
- **Configuration generation**: Generate random valid configurations
- **Invariant checking**: Ensure configuration invariants hold
- **Roundtrip testing**: Test serialization/deserialization
- **Stress testing**: Test with large numbers of arguments

---

*Status*: ✅ Implemented  
*Owner*: JavaScript Minifier Team  
*Last Updated*: 2025-08-25