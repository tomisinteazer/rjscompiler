//! # Error Recovery Module
//!
//! This module provides error recovery strategies for the JavaScript parser.
//! It helps the parser continue parsing even when syntax errors are encountered,
//! allowing for better error reporting and IDE support.

use crate::parser::ParseError;
use serde::{Deserialize, Serialize};

/// Error recovery strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryStrategy {
    /// Skip to the next statement boundary
    SkipToStatement,
    /// Skip to the next expression boundary
    SkipToExpression,
    /// Insert a missing token
    InsertToken(String),
    /// Replace an invalid token
    ReplaceToken(String, String),
}

/// Error recovery context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryContext {
    /// The error that triggered recovery
    pub error: ParseError,
    /// The recovery strategy used
    pub strategy: RecoveryStrategy,
    /// Whether recovery was successful
    pub successful: bool,
    /// Additional context information
    pub context: String,
}

/// Error recovery engine
pub struct ErrorRecovery {
    /// Maximum number of errors to recover from
    max_errors: usize,
    /// Current number of recovered errors
    recovered_count: usize,
    /// Recovery history
    recovery_history: Vec<RecoveryContext>,
}

impl ErrorRecovery {
    /// Create a new error recovery engine
    pub fn new(max_errors: usize) -> Self {
        Self {
            max_errors,
            recovered_count: 0,
            recovery_history: Vec::new(),
        }
    }

    /// Attempt to recover from a parse error
    pub fn recover_from_error(&mut self, error: ParseError, source: &str) -> Option<RecoveryStrategy> {
        if self.recovered_count >= self.max_errors {
            return None;
        }

        let strategy = self.determine_recovery_strategy(&error, source);
        
        if let Some(ref strategy) = strategy {
            let context = RecoveryContext {
                error: error.clone(),
                strategy: strategy.clone(),
                successful: true,
                context: "Error recovery attempted".to_string(),
            };
            
            self.recovery_history.push(context);
            self.recovered_count += 1;
        }

        strategy
    }

    /// Determine the best recovery strategy for an error
    fn determine_recovery_strategy(&self, error: &ParseError, _source: &str) -> Option<RecoveryStrategy> {
        match error {
            ParseError::UnterminatedString { .. } => {
                Some(RecoveryStrategy::InsertToken("\"".to_string()))
            }
            ParseError::UnexpectedToken { token, .. } => {
                if token == "=" {
                    Some(RecoveryStrategy::InsertToken("identifier".to_string()))
                } else {
                    Some(RecoveryStrategy::SkipToStatement)
                }
            }
            ParseError::SyntaxError { .. } => {
                Some(RecoveryStrategy::SkipToStatement)
            }
            _ => Some(RecoveryStrategy::SkipToStatement),
        }
    }

    /// Get recovery statistics
    pub fn get_recovery_stats(&self) -> RecoveryStats {
        RecoveryStats {
            total_errors: self.recovered_count,
            successful_recoveries: self.recovery_history.iter()
                .filter(|ctx| ctx.successful)
                .count(),
            max_errors: self.max_errors,
            recovery_rate: if self.recovered_count > 0 {
                self.recovery_history.iter()
                    .filter(|ctx| ctx.successful)
                    .count() as f64 / self.recovered_count as f64
            } else {
                0.0
            },
        }
    }

    /// Reset the recovery engine
    pub fn reset(&mut self) {
        self.recovered_count = 0;
        self.recovery_history.clear();
    }
}

/// Recovery statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryStats {
    /// Total number of errors encountered
    pub total_errors: usize,
    /// Number of successful recoveries
    pub successful_recoveries: usize,
    /// Maximum errors allowed
    pub max_errors: usize,
    /// Recovery success rate (0.0 to 1.0)
    pub recovery_rate: f64,
}

/// Helper function to find the next statement boundary
pub fn find_next_statement_boundary(source: &str, offset: usize) -> Option<usize> {
    let chars: Vec<char> = source.chars().collect();
    let mut i = offset;
    
    while i < chars.len() {
        match chars[i] {
            ';' | '\n' => return Some(i + 1),
            '{' => {
                // Skip to matching closing brace
                let mut brace_count = 1;
                i += 1;
                while i < chars.len() && brace_count > 0 {
                    match chars[i] {
                        '{' => brace_count += 1,
                        '}' => brace_count -= 1,
                        _ => {}
                    }
                    i += 1;
                }
                return Some(i);
            }
            _ => i += 1,
        }
    }
    
    None
}

/// Helper function to find the next expression boundary
pub fn find_next_expression_boundary(source: &str, offset: usize) -> Option<usize> {
    let chars: Vec<char> = source.chars().collect();
    let mut i = offset;
    let mut paren_count = 0;
    
    while i < chars.len() {
        match chars[i] {
            '(' => paren_count += 1,
            ')' => {
                paren_count -= 1;
                if paren_count <= 0 {
                    return Some(i + 1);
                }
            }
            ',' | ';' if paren_count == 0 => return Some(i),
            _ => {}
        }
        i += 1;
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_recovery_creation() {
        let recovery = ErrorRecovery::new(10);
        assert_eq!(recovery.max_errors, 10);
        assert_eq!(recovery.recovered_count, 0);
    }

    #[test]
    fn test_find_statement_boundary() {
        let source = "let x = 5; let y = 10;";
        let boundary = find_next_statement_boundary(source, 0);
        assert_eq!(boundary, Some(10)); // After first semicolon
    }

    #[test]
    fn test_find_expression_boundary() {
        let source = "func(a, b, c)";
        let boundary = find_next_expression_boundary(source, 5);
        assert_eq!(boundary, Some(8)); // After first comma
    }
}