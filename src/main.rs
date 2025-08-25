//! # RJS Compiler
//!
//! A Rust-based JavaScript compiler that provides fast and reliable JavaScript compilation.
//! This CLI tool processes JavaScript source files and transforms them according to
//! specified compilation rules and optimizations.
//!
//! ## Features
//!
//! - Fast compilation using Rust's performance characteristics
//! - Verbose output for debugging compilation processes
//! - Comprehensive error handling and reporting
//! - Cross-platform compatibility
//!
//! ## Usage
//!
//! ```bash
//! rjs-compiler [OPTIONS] <FILE>
//! ```
//!
//! For more information, run `rjs-compiler --help`.

use std::path::PathBuf;
use std::process;

use clap::{Arg, Command};

mod parser;
mod analyzer;
mod transformer;
mod generator;

/// Application version constant
const VERSION: &str = "0.1.0";

/// Application name constant
const APP_NAME: &str = "rjs-compiler";

/// Configuration structure for the compiler
#[derive(Debug, Clone)]
struct CompilerConfig {
    /// Input file path to compile
    input_file: Option<PathBuf>,
    /// Output file path for minified code
    output_file: Option<PathBuf>,
    /// Enable verbose output
    verbose: bool,
}

/// Custom error types for the compiler
#[derive(Debug, thiserror::Error)]
enum CompilerError {
    #[error("Input file not specified")]
    MissingInputFile,
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),
    #[error("Parse error: {0}")]
    ParseError(String),
}

/// Result type alias for compiler operations
type CompilerResult<T> = Result<T, CompilerError>;

/// Entry point for the RJS Compiler application.
///
/// This function sets up command-line argument parsing, initializes the compiler
/// configuration, and orchestrates the compilation process.
fn main() {
    if let Err(error) = run_compiler() {
        eprintln!("Error: {}", error);
        process::exit(1);
    }
}

/// Main application logic separated from main() for better error handling.
///
/// # Returns
///
/// Returns `Ok(())` on successful compilation, or a `CompilerError` on failure.
///
/// # Examples
///
/// ```rust,no_run
/// // This is called internally by main()
/// match run_compiler() {
///     Ok(()) => println!("Compilation successful"),
///     Err(e) => eprintln!("Compilation failed: {}", e),
/// }
/// ```
fn run_compiler() -> CompilerResult<()> {
    let config = parse_command_line_arguments()?;
    
    display_welcome_message();
    
    if config.verbose {
        display_verbose_info(&config);
    }
    
    match config.input_file {
        Some(ref file_path) => compile_file(file_path, &config),
        None => {
            display_usage_information();
            Err(CompilerError::MissingInputFile)
        }
    }
}

/// Parses command-line arguments and returns a compiler configuration.
///
/// # Returns
///
/// Returns a `CompilerResult<CompilerConfig>` containing the parsed configuration
/// or an error if argument parsing fails.
///
/// # Errors
///
/// This function currently doesn't return errors but is designed to handle
/// future validation requirements.
fn parse_command_line_arguments() -> CompilerResult<CompilerConfig> {
    let matches = Command::new(APP_NAME)
        .version(VERSION)
        .author("RJS Compiler Team <team@rjscompiler.dev>")
        .about("RJS Compiler - A Rust-based JavaScript compiler")
        .long_about(
            "A high-performance JavaScript compiler built with Rust. \n\n\
             This tool processes JavaScript source files and applies various \n\
             compilation optimizations and transformations."
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(clap::ArgAction::SetTrue)
                .help("Enable verbose output for debugging")
                .long_help(
                    "Enable verbose output mode which provides detailed information \n\
                     about the compilation process, including intermediate steps \n\
                     and performance metrics."
                ),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .help("Output file for minified JavaScript")
                .long_help(
                    "Path to the output file where the minified JavaScript \n\
                     will be saved. If not specified, output will be printed \n\
                     to stdout. File will be created if it doesn't exist."
                )
                .value_name("OUTPUT_FILE")
                .value_parser(clap::value_parser!(PathBuf)),
        )
        .arg(
            Arg::new("input")
                .help("Input JavaScript file to compile")
                .long_help(
                    "Path to the JavaScript source file that will be compiled. \n\
                     The file must exist and be readable."
                )
                .value_name("FILE")
                .value_parser(clap::value_parser!(PathBuf))
                .index(1),
        )
        .get_matches();

    let input_file = matches.get_one::<PathBuf>("input").cloned();
    let output_file = matches.get_one::<PathBuf>("output").cloned();
    let verbose = matches.get_flag("verbose");

    Ok(CompilerConfig {
        input_file,
        output_file,
        verbose,
    })
}

/// Displays the welcome message for the application.
///
/// This function prints the application greeting and version information
/// in a user-friendly format.
fn display_welcome_message() {
    println!("🦀 Hello Rust!");
    println!("Welcome to RJS Compiler v{}", VERSION);
}

/// Displays verbose information about the current configuration.
///
/// # Arguments
///
/// * `config` - The compiler configuration containing settings to display
///
/// # Examples
///
/// ```rust,no_run
/// let config = CompilerConfig {
///     input_file: Some(PathBuf::from("test.js")),
///     output_file: Some(PathBuf::from("build.js")),
///     verbose: true,
/// };
/// display_verbose_info(&config);
/// ```
fn display_verbose_info(config: &CompilerConfig) {
    println!("🔍 Verbose mode enabled");
    println!("📋 Configuration:");
    
    if let Some(ref input_path) = config.input_file {
        println!("   📁 Input file: {}", input_path.display());
    }
    
    if let Some(ref output_path) = config.output_file {
        println!("   📄 Output file: {}", output_path.display());
    } else {
        println!("   📄 Output file: stdout (console)");
    }
    
    println!("   🔧 Verbose output: {}", config.verbose);
}

/// Displays usage information when no input file is provided.
///
/// This function provides helpful guidance to users about how to use
/// the compiler correctly.
fn display_usage_information() {
    println!("💡 Usage: {} [OPTIONS] <FILE>", APP_NAME);
    println!("   Use --help for more information");
    println!("   Example: {} --verbose my_script.js", APP_NAME);
    println!("   Example: {} -o build.js my_script.js", APP_NAME);
}

/// Compiles the specified JavaScript file and saves the minified output.
///
/// # Arguments
///
/// * `file_path` - Path to the JavaScript file to compile
/// * `config` - Compiler configuration settings including output file destination
///
/// # Returns
///
/// Returns `Ok(())` on successful compilation, or a `CompilerError` on failure.
///
/// # Errors
///
/// Returns `CompilerError::FileNotFound` if the input file doesn't exist.
/// Returns `CompilerError::ParseError` if file writing fails.
///
/// # Output Behavior
///
/// If an output file is specified in config, the minified code is saved there.
/// If no output file is specified, defaults to 'build.js' in the input file's directory.
///
/// # Examples
///
/// ```rust,no_run
/// use std::path::PathBuf;
/// 
/// let file_path = PathBuf::from("example.js");
/// let config = CompilerConfig {
///     input_file: Some(file_path.clone()),
///     output_file: Some(PathBuf::from("build.js")),
///     verbose: false,
/// };
/// 
/// match compile_file(&file_path, &config) {
///     Ok(()) => println!("Compilation successful"),
///     Err(e) => eprintln!("Compilation failed: {}", e),
/// }
/// ```
fn compile_file(file_path: &PathBuf, config: &CompilerConfig) -> CompilerResult<()> {
    if config.verbose {
        println!("📂 Processing input file: {}", file_path.display());
    }
    
    // Validate that the file exists
    if !file_path.exists() {
        return Err(CompilerError::FileNotFound(file_path.clone()));
    }
    
    if config.verbose {
        println!("✅ Input file validation passed");
        println!("🚀 Starting compilation process...");
    }
    
    // Read the file content
    let source_code = std::fs::read_to_string(file_path)
        .map_err(|_| CompilerError::FileNotFound(file_path.clone()))?;
    
    if config.verbose {
        println!("📄 Read {} bytes from file", source_code.len());
    }
    
    // Parse the JavaScript file
    let parser_config = parser::ParserConfig {
        preserve_trivia: config.verbose, // Enable trivia preservation in verbose mode
        ..parser::ParserConfig::default()
    };
    let parse_result = parser::parse_js(&source_code, &file_path.to_string_lossy(), &parser_config);
    
    if config.verbose {
        println!("🔍 Phase 1: Parsing completed");
    }
    
    // Check for parsing errors
    if !parse_result.errors.is_empty() {
        eprintln!("❌ Parsing errors found:");
        for error in &parse_result.errors {
            eprintln!("   {}", error);
        }
        return Err(CompilerError::ParseError(format!(
            "Found {} parsing errors", 
            parse_result.errors.len()
        )));
    }
    
    // Extract the AST
    let ast = parse_result.ast.ok_or_else(|| {
        CompilerError::ParseError("No AST generated despite no errors".to_string())
    })?;
    
    if config.verbose {
        println!("📊 AST Statistics:");
        println!("   📋 Statements: {}", ast.body.len());
        println!("   📘 Source type: {:?}", ast.source_type);
        
        // Display trivia information if available
        if let Some(ref trivia) = parse_result.trivia {
            println!("📝 Trivia Preserved:");
            println!("   💬 Line comments: {}", trivia.line_comments.len());
            println!("   💬 Block comments: {}", trivia.block_comments.len());
            println!("   ⬜ Leading whitespace: {}", trivia.leading_whitespace.len());
            println!("   ⬜ Trailing whitespace: {}", trivia.trailing_whitespace.len());
            
            // Show first few comments for debugging
            if !trivia.line_comments.is_empty() {
                println!("   🗺 Sample line comments:");
                for (i, comment) in trivia.line_comments.iter().take(3).enumerate() {
                    println!("     {}. '{}' (pos: {}-{})", 
                        i + 1, comment.text, comment.span.start, comment.span.end);
                }
            }
            
            if !trivia.block_comments.is_empty() {
                println!("   🗺 Sample block comments:");
                for (i, comment) in trivia.block_comments.iter().take(3).enumerate() {
                    println!("     {}. '{}' (pos: {}-{})", 
                        i + 1, comment.text, comment.span.start, comment.span.end);
                }
            }
        }
        
        // Pretty print AST in JSON format for debugging
        if let Ok(ast_json) = serde_json::to_string_pretty(&ast) {
            println!("🌳 AST Structure (JSON):");
            // Limit output to first 1000 characters for readability
            let truncated = if ast_json.len() > 1000 {
                format!("{}...\n(truncated)", &ast_json[..1000])
            } else {
                ast_json
            };
            println!("{}", truncated);
        }
    }
    
    // Phase 3: Semantic Analysis
    if config.verbose {
        println!("🔍 Phase 3: Starting semantic analysis...");
    }
    
    let analyzer_config = analyzer::AnalyzerConfig {
        verbose: config.verbose,
        preserve_exports: true,
        aggressive_optimization: false,
        strict_mode: true,
    };
    
    let analysis_result = analyzer::analyze_ast(&ast, &analyzer_config)
        .map_err(|e| CompilerError::ParseError(format!("Analysis failed: {}", e)))?;
    
    if config.verbose {
        println!("📊 Analysis Results:");
        println!("   🏗️  Scopes analyzed: {}", analysis_result.metadata.scope_count);
        println!("   🏷️  Symbols found: {}", analysis_result.metadata.symbol_count);
        println!("   🔗 Closure captures: {}", analysis_result.metadata.capture_count);
        println!("   📤 Export symbols: {}", analysis_result.metadata.export_count);
        println!("   ⏱️  Analysis time: {}ms", analysis_result.metadata.analysis_time_ms);
        
        // Display unsafe scopes
        if !analysis_result.semantic_flags.unsafe_scopes.is_empty() {
            println!("   ⚠️  Unsafe scopes detected: {}", analysis_result.semantic_flags.unsafe_scopes.len());
            for (scope_id, reason) in &analysis_result.semantic_flags.unsafe_scopes {
                println!("     Scope {}: {:?}", scope_id, reason);
            }
        }
        
        // Display symbol statistics
        let renamable_symbols = analysis_result.symbol_table.symbols.values()
            .filter(|s| s.is_renamable)
            .count();
        let captured_symbols = analysis_result.symbol_table.symbols.values()
            .filter(|s| s.is_captured)
            .count();
        
        println!("   ✅ Renamable symbols: {}", renamable_symbols);
        println!("   📎 Captured symbols: {}", captured_symbols);
    }
    
    // Phase 4: Transformation
    if config.verbose {
        println!("🔄 Phase 4: Starting transformation...");
    }
    
    let _transformer_config = transformer::TransformerConfig {
        enable_identifier_renaming: true,
        enable_dead_code_elimination: true,
        enable_expression_simplification: true,
        enable_property_minification: true,
        enable_function_minification: true,
        enable_rollback: true,
        verbose: config.verbose,
        aggressive_optimization: false,
    };
    
    let transformation_result = transformer::transform_ast(ast, analysis_result)
        .map_err(|e| CompilerError::ParseError(format!("Transformation failed: {}", e)))?;
    
    if config.verbose {
        println!("📊 Transformation Results:");
        println!("   🏷️  Identifiers renamed: {}", transformation_result.stats.identifiers_renamed);
        println!("   🗑️  Dead statements removed: {}", transformation_result.stats.dead_statements_removed);
        println!("   🔧 Expressions simplified: {}", transformation_result.stats.expressions_simplified);
        println!("   🏠 Properties renamed: {}", transformation_result.stats.properties_renamed);
        println!("   📎 Functions inlined: {}", transformation_result.stats.functions_inlined);
        println!("   ⏱️  Transformation time: {}ms", transformation_result.stats.transformation_time_ms);
        
        if !transformation_result.warnings.is_empty() {
            println!("   ⚠️  Warnings:");
            for warning in &transformation_result.warnings {
                println!("     {}", warning);
            }
        }
        
        if !transformation_result.identifier_mapping.is_empty() {
            println!("   🔄 Identifier mappings:");
            for (original, renamed) in transformation_result.identifier_mapping.iter().take(5) {
                println!("     {} -> {}", original, renamed);
            }
            if transformation_result.identifier_mapping.len() > 5 {
                println!("     ... and {} more", transformation_result.identifier_mapping.len() - 5);
            }
        }
        
        println!("   🎯 Statements processed: {}", transformation_result.transformed_ast.body.len());
    }
    
    // Phase 5: Code Generation
    if config.verbose {
        println!("🏗️ Phase 5: Starting code generation...");
    }
    
    let generator_config = generator::GeneratorConfig {
        format: generator::OutputFormat::Compact,
        semicolon: generator::SemicolonStrategy::Auto,
        quote: generator::QuoteStrategy::Auto,
        preserve_comments: generator::CommentPreservation::None,
        source_map: generator::SourceMapMode::None,
        ..generator::GeneratorConfig::default()
    };
    
    let generator = generator::Generator::new(generator_config);
    let generation_result = generator.generate(&transformation_result.transformed_ast, Some(&source_code))
        .map_err(|e| CompilerError::ParseError(format!("Code generation failed: {}", e)))?;
    
    if config.verbose {
        println!("📊 Generation Results:");
        println!("   📏 Original size: {} bytes", generation_result.diagnostics.original_size);
        println!("   📏 Generated size: {} bytes", generation_result.diagnostics.generated_size);
        println!("   📉 Compression ratio: {:.1}%", generation_result.diagnostics.compression_ratio * 100.0);
        println!("   ⏱️  Generation time: {:.2}ms", generation_result.diagnostics.generation_time_ms);
        
        if generation_result.diagnostics.warning_count > 0 {
            println!("   ⚠️  Generation warnings: {}", generation_result.diagnostics.warning_count);
            for warning in &generation_result.diagnostics.warnings {
                println!("     {}", warning);
            }
        }
    }
    
    // Determine output destination
    let output_path = config.output_file.as_ref()
        .cloned()
        .unwrap_or_else(|| {
            // Default to build.js in the same directory as input file
            if let Some(ref input_path) = config.input_file {
                let mut output_path = input_path.clone();
                output_path.set_file_name("build.js");
                output_path
            } else {
                PathBuf::from("build.js")
            }
        });
    
    // Write the minified code to file
    std::fs::write(&output_path, &generation_result.code)
        .map_err(|e| CompilerError::ParseError(format!("Failed to write output file '{}': {}", output_path.display(), e)))?;
    
    if config.verbose {
        println!("💾 Output written to: {}", output_path.display());
        println!("🎯 Generated Code Preview:");
        // Show a preview of the generated code (first 200 characters)
        let preview = if generation_result.code.len() > 200 {
            format!("{}...", &generation_result.code[..200])
        } else {
            generation_result.code.clone()
        };
        println!("{}", preview);
    } else {
        // In non-verbose mode, just show the output file location
        println!("📄 Minified JavaScript saved to: {}", output_path.display());
    }
    
    println!("✅ Compilation completed successfully!");
    
    if config.verbose {
        println!("📊 Compilation statistics:");
        println!("   ⏱️  Total file size reduction: {:.1}%", generation_result.diagnostics.compression_ratio * 100.0);
        println!("   📁 Input: {} -> 📄 Output: {}", 
            config.input_file.as_ref().map(|p| p.display().to_string()).unwrap_or_else(|| "<unknown>".to_string()),
            output_path.display());
    }
    
    Ok(())
}

/// Simulates the remaining compilation process for demonstration purposes.
///
/// This function represents where the code generation logic will be implemented
/// in future iterations. The transformation phase is now complete.
///
/// # Arguments
///
/// * `config` - Compiler configuration for controlling compilation behavior
///
/// # Returns
///
/// Returns `Ok(())` on successful simulation, or a `CompilerError` on failure.
///
/// # Note
///
/// This is a placeholder function for Phase 5 (code generation).
fn simulate_compilation_process(config: &CompilerConfig) -> CompilerResult<()> {
    if config.verbose {
        println!("🔄 Phase 5: Code generation (TODO)");
    }
    
    // Simulate successful compilation
    // In a real implementation, this would contain the code generation pipeline
    Ok(())
}
