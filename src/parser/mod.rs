//! # JavaScript Parser Module
//!
//! This module provides JavaScript parsing functionality using the OXC parser.
//! It converts JavaScript source code into an Abstract Syntax Tree (AST) with
//! preserved trivia/comments for accurate reconstruction.
//!
//! ## Features
//!
//! - **Fast Parsing**: Uses OXC parser for high-performance parsing
//! - **ES6+ Support**: Handles modern JavaScript syntax features
//! - **Error Handling**: Provides meaningful syntax error messages with position info
//! - **Trivia Preservation**: Maintains comments and whitespace for reconstruction
//!
//! ## Usage
//!
//! ```rust
//! use crate::parser::{parse_js, ParserConfig};
//!
//! let source = "let x = 5;";
//! let config = ParserConfig::default();
//! let result = parse_js(source, "example.js", &config);
//! ```

use oxc_allocator::Allocator;
use oxc_ast::ast::Program;
use oxc_parser::{Parser, ParserReturn};
use oxc_span::SourceType;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod ast_types;
pub mod error_recovery;

#[cfg(test)]
mod tests;

/// Configuration for the JavaScript parser
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParserConfig {
    /// Whether to preserve comments and trivia
    pub preserve_trivia: bool,
    /// Whether to recover from syntax errors
    pub error_recovery: bool,
    /// Source type (Script, Module, TypeScript, etc.)
    pub source_type: SourceTypeConfig,
}

/// Source type configuration for parsing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SourceTypeConfig {
    /// JavaScript script
    Script,
    /// ES6 module
    Module,
    /// TypeScript (future enhancement)
    TypeScript,
}

/// Errors that can occur during parsing
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum ParseError {
    #[error("Syntax error at line {line}, column {column}: {message}")]
    SyntaxError {
        message: String,
        line: u32,
        column: u32,
        span: Option<SourceSpan>,
    },
    #[error("Unterminated string literal at line {line}, column {column}")]
    UnterminatedString { line: u32, column: u32 },
    #[error("Unexpected token '{token}' at line {line}, column {column}")]
    UnexpectedToken {
        token: String,
        line: u32,
        column: u32,
    },
    #[error("Invalid regular expression: {message}")]
    InvalidRegex { message: String },
    #[error("Internal parser error: {message}")]
    InternalError { message: String },
}

/// Source position information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceSpan {
    pub start: u32,
    pub end: u32,
}

/// Parse result containing the AST or multiple errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseResult {
    /// The parsed AST (if successful)
    pub ast: Option<ast_types::Program>,
    /// Any errors encountered during parsing
    pub errors: Vec<ParseError>,
    /// Source trivia (comments, whitespace) if preserved
    pub trivia: Option<Trivia>,
}

/// Trivia information (comments and whitespace)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trivia {
    /// Line comments
    pub line_comments: Vec<Comment>,
    /// Block comments
    pub block_comments: Vec<Comment>,
    /// Leading whitespace
    pub leading_whitespace: Vec<Whitespace>,
    /// Trailing whitespace
    pub trailing_whitespace: Vec<Whitespace>,
}

/// Comment information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    /// Comment text (without // or /* */)
    pub text: String,
    /// Source position
    pub span: SourceSpan,
    /// Whether it's a line comment or block comment
    pub kind: CommentKind,
}

/// Type of comment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommentKind {
    Line,
    Block,
}

/// Whitespace information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Whitespace {
    /// Whitespace text
    pub text: String,
    /// Source position
    pub span: SourceSpan,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            preserve_trivia: true,
            error_recovery: true,
            source_type: SourceTypeConfig::Module,
        }
    }
}

impl From<SourceTypeConfig> for SourceType {
    fn from(config: SourceTypeConfig) -> Self {
        match config {
            SourceTypeConfig::Script => SourceType::default(),
            SourceTypeConfig::Module => SourceType::default().with_module(true),
            SourceTypeConfig::TypeScript => SourceType::default().with_typescript(true),
        }
    }
}

/// Main parsing function that converts JavaScript source code into an AST.
///
/// # Arguments
///
/// * `source` - The JavaScript source code to parse
/// * `filename` - The filename for error reporting
/// * `config` - Parser configuration options
///
/// # Returns
///
/// Returns a `ParseResult` containing the AST and any errors encountered.
///
/// # Examples
///
/// ```rust
/// use crate::parser::{parse_js, ParserConfig};
///
/// let source = "let x = 5;";
/// let config = ParserConfig::default();
/// let result = parse_js(source, "example.js", &config);
///
/// if result.errors.is_empty() {
///     println!("Parsing successful!");
/// }
/// ```
pub fn parse_js(source: &str, filename: &str, config: &ParserConfig) -> ParseResult {
    let allocator = Allocator::default();
    let source_type = SourceType::from(config.source_type.clone());
    
    let ret = Parser::new(&allocator, source, source_type).parse();
    
    convert_parser_result(ret, source, filename, config)
}

/// Converts OXC parser result to our internal ParseResult format
fn convert_parser_result(
    ret: ParserReturn<'_>,
    source: &str,
    _filename: &str,
    config: &ParserConfig,
) -> ParseResult {
    let mut errors = Vec::new();
    
    // Convert OXC errors to our error format
    for error in ret.errors {
        let (line, column) = (1, 1); // TODO: Extract from diagnostic
        
        let parse_error = ParseError::SyntaxError {
            message: format!("{:?}", error),
            line,
            column,
            span: None,
        };
        errors.push(parse_error);
    }
    
    // Convert AST if parsing was successful
    let ast = if errors.is_empty() {
        Some(ast_types::Program::from_oxc(&ret.program))
    } else {
        None
    };
    
    // Extract trivia if configured
    let trivia = if config.preserve_trivia {
        Some(extract_trivia(source, &ret.program))
    } else {
        None
    };
    
    ParseResult { ast, errors, trivia }
}

/// Calculates line and column numbers from a byte offset
#[allow(dead_code)]
fn get_line_column(source: &str, offset: u32) -> (u32, u32) {
    let mut line = 1;
    let mut column = 1;
    
    for (i, ch) in source.char_indices() {
        if i >= offset as usize {
            break;
        }
        
        if ch == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    
    (line, column)
}

/// Extracts trivia information from the source code
fn extract_trivia(source: &str, _program: &Program) -> Trivia {
    let mut line_comments = Vec::new();
    let mut block_comments = Vec::new();
    let mut leading_whitespace = Vec::new();
    let mut trailing_whitespace = Vec::new();
    
    let mut pos = 0;
    let chars: Vec<char> = source.chars().collect();
    let mut in_string = false;
    let mut string_delimiter = '\0';
    let mut escaped = false;
    
    while pos < chars.len() {
        let ch = chars[pos];
        
        // Handle string context
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == string_delimiter {
                in_string = false;
                string_delimiter = '\0';
            }
            pos += 1;
            continue;
        }
        
        match ch {
            // Start of string
            '"' | '\'' | '`' => {
                in_string = true;
                string_delimiter = ch;
                pos += 1;
            }
            // Handle line comments (only when not in string)
            '/' if pos + 1 < chars.len() && chars[pos + 1] == '/' => {
                let start = pos;
                pos += 2; // Skip '//'
                
                // Extract comment text until end of line
                let mut comment_text = String::new();
                while pos < chars.len() && chars[pos] != '\n' {
                    comment_text.push(chars[pos]);
                    pos += 1;
                }
                
                line_comments.push(Comment {
                    text: comment_text.trim().to_string(),
                    span: SourceSpan {
                        start: start as u32,
                        end: pos as u32,
                    },
                    kind: CommentKind::Line,
                });
                
                if pos < chars.len() {
                    pos += 1; // Skip newline
                }
            }
            // Handle block comments (only when not in string)
            '/' if pos + 1 < chars.len() && chars[pos + 1] == '*' => {
                let start = pos;
                pos += 2; // Skip '/*'
                
                let mut comment_text = String::new();
                let mut found_end = false;
                
                while pos + 1 < chars.len() {
                    if chars[pos] == '*' && chars[pos + 1] == '/' {
                        pos += 2; // Skip '*/'
                        found_end = true;
                        break;
                    }
                    comment_text.push(chars[pos]);
                    pos += 1;
                }
                
                if found_end {
                    block_comments.push(Comment {
                        text: comment_text.trim().to_string(),
                        span: SourceSpan {
                            start: start as u32,
                            end: pos as u32,
                        },
                        kind: CommentKind::Block,
                    });
                }
            }
            // Handle whitespace
            ' ' | '\t' | '\r' | '\n' => {
                let start = pos;
                let mut whitespace_text = String::new();
                
                // Collect consecutive whitespace
                while pos < chars.len() && matches!(chars[pos], ' ' | '\t' | '\r' | '\n') {
                    whitespace_text.push(chars[pos]);
                    pos += 1;
                }
                
                // Determine if it's leading or trailing based on context
                // For simplicity, consider whitespace at start of line as leading
                let is_leading = whitespace_text.contains('\n') || start == 0;
                
                let whitespace = Whitespace {
                    text: whitespace_text,
                    span: SourceSpan {
                        start: start as u32,
                        end: pos as u32,
                    },
                };
                
                if is_leading {
                    leading_whitespace.push(whitespace);
                } else {
                    trailing_whitespace.push(whitespace);
                }
            }
            _ => {
                pos += 1;
            }
        }
    }
    
    Trivia {
        line_comments,
        block_comments,
        leading_whitespace,
        trailing_whitespace,
    }
}

/// Helper function to create a simple syntax error
#[allow(dead_code)]
pub fn create_syntax_error(message: &str, line: u32, column: u32) -> ParseError {
    ParseError::SyntaxError {
        message: message.to_string(),
        line,
        column,
        span: None,
    }
}

/// Helper function to create an unexpected token error
#[allow(dead_code)]
pub fn create_unexpected_token_error(token: &str, line: u32, column: u32) -> ParseError {
    ParseError::UnexpectedToken {
        token: token.to_string(),
        line,
        column,
    }
}

/// Helper function to create an unterminated string error
#[allow(dead_code)]
pub fn create_unterminated_string_error(line: u32, column: u32) -> ParseError {
    ParseError::UnterminatedString { line, column }
}