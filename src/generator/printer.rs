//! # Printer Component (Component 12)
//!
//! The printer component walks the AST and emits tokens with minimal bytes while preserving semantics.
//! It handles operator precedence, ASI hazards, string/number canonicalization, and tracks positions
//! for source map generation.

use crate::generator::{GeneratorConfig, GeneratorResult};
use crate::parser::ast_types::*;

/// Operator precedence levels (higher number = higher precedence)
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Precedence {
    Sequence = 1,        // ,
    Assignment = 3,      // =, +=, -=, etc.
    Conditional = 4,     // ?:
    LogicalOr = 5,       // ||
    LogicalAnd = 6,      // &&
    BitwiseOr = 7,       // |
    BitwiseXor = 8,      // ^
    BitwiseAnd = 9,      // &
    Equality = 10,       // ==, !=, ===, !==
    Relational = 11,     // <, <=, >, >=, in, instanceof
    Shift = 12,          // <<, >>, >>>
    Additive = 13,       // +, -
    Multiplicative = 14, // *, /, %
    Exponentiation = 15, // ** (right-associative)
    Unary = 16,          // !, ~, +, -, typeof, void, delete, await
    Postfix = 17,        // ++, --
    Member = 19,         // ., [], ?., ?.[], ?.(), ()
}

/// Associativity for operators
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Associativity {
    Left,
    Right,
    None,
}

/// Token types for ASI detection
#[derive(Debug, Clone, Copy, PartialEq)]
enum TokenType {
    Identifier,
    Number,
    String,
    CloseParen,
    CloseBracket,
    Increment,
    Decrement,
    Regex,
}

/// Printer implementation for Component 12
pub struct Printer {
    config: GeneratorConfig,
    output: String,
    warnings: Vec<String>,
    prev_token: Option<TokenType>,
    indent_level: usize,
    /// Pre-allocated string buffer for performance
    string_buffer: String,
    /// Cached indent strings for performance
    indent_cache: Vec<String>,
    /// Performance metrics
    chars_written: usize,
}

impl Printer {
    /// Create a new printer with the given configuration
    pub fn new(config: &GeneratorConfig) -> Self {
        let mut printer = Self {
            config: config.clone(),
            output: String::new(),
            warnings: Vec::new(),
            prev_token: None,
            indent_level: 0,
            string_buffer: String::with_capacity(1024), // Pre-allocate buffer
            indent_cache: Vec::new(),
            chars_written: 0,
        };
        
        // Pre-populate indent cache for performance
        printer.populate_indent_cache();
        
        // Estimate and pre-allocate output capacity
        printer.optimize_output_capacity();
        
        printer
    }

    /// Print a complete program
    pub fn print_program(&mut self, program: &Program) -> GeneratorResult<String> {
        self.output.clear();
        self.warnings.clear();
        
        // Validate the program structure first
        self.validate_program(program)?;
        
        for (i, stmt) in program.body.iter().enumerate() {
            if i > 0 {
                self.print_statement_separator()?;
            }
            
            // Check for ASI hazards
            self.check_asi_hazard(stmt)?;
            
            // Check memory limits periodically
            if i % 100 == 0 {
                self.check_memory_limits()?;
            }
            
            self.print_statement(stmt)?;
        }
        
        // Final memory check
        self.check_memory_limits()?;
        
        // Add final newline for readable/pretty formats
        match self.config.format {
            crate::generator::OutputFormat::Readable | crate::generator::OutputFormat::Pretty => {
                if !program.body.is_empty() {
                    self.print_newline_if_needed()?;
                }
            }
            _ => {}
        }
        
        Ok(self.output.clone())
    }

    /// Get warnings generated during printing
    pub fn get_warnings(&self) -> Vec<String> {
        self.warnings.clone()
    }

    /// Print a statement
    fn print_statement(&mut self, stmt: &Statement) -> GeneratorResult<()> {
        match stmt {
            Statement::VariableDeclaration { declarations, kind } => {
                self.print_variable_declaration(declarations, kind)
            }
            Statement::FunctionDeclaration { id, params, body, is_async, is_generator } => {
                self.print_function_declaration(id, params, body, *is_async, *is_generator)
            }
            Statement::ExpressionStatement { expression } => {
                self.print_expression_statement(expression)
            }
            Statement::BlockStatement { body } => {
                self.print_block_statement(body)
            }
            Statement::ReturnStatement { argument } => {
                self.print_return_statement(argument)
            }
            _ => {
                // TODO: Implement remaining statement types
                self.write("/* STMT */")?;
                Ok(())
            }
        }
    }

    /// Print a variable declaration
    fn print_variable_declaration(
        &mut self,
        declarations: &[VariableDeclarator],
        kind: &VariableDeclarationKind,
    ) -> GeneratorResult<()> {
        match kind {
            VariableDeclarationKind::Var => self.write("var")?,
            VariableDeclarationKind::Let => self.write("let")?,
            VariableDeclarationKind::Const => self.write("const")?,
        }

        self.print_space_if_needed()?;

        for (i, declarator) in declarations.iter().enumerate() {
            if i > 0 {
                self.write(",")?;
                self.print_space_if_needed()?;
            }
            self.print_variable_declarator(declarator)?;
        }

        self.print_semicolon_if_needed()?;
        Ok(())
    }

    /// Print a variable declarator
    fn print_variable_declarator(&mut self, declarator: &VariableDeclarator) -> GeneratorResult<()> {
        self.print_pattern(&declarator.id)?;
        
        if let Some(init) = &declarator.init {
            self.print_assignment_operator()?;
            self.print_expression(init, Precedence::Assignment)?;
        }
        
        Ok(())
    }

    /// Print a function declaration
    fn print_function_declaration(
        &mut self,
        id: &Option<Identifier>,
        params: &[Pattern],
        body: &BlockStatement,
        is_async: bool,
        is_generator: bool,
    ) -> GeneratorResult<()> {
        if is_async {
            self.write("async")?;
            self.print_space_if_needed()?;
        }

        self.write("function")?;

        if is_generator {
            self.write("*")?;
        }

        if let Some(id) = id {
            self.print_space_if_needed()?;
            self.print_identifier(id)?;
        }

        self.write("(")?;
        self.print_parameter_list(params)?;
        self.write(")")?;

        self.print_block_statement_body(&body.body)?;
        Ok(())
    }

    /// Print an expression statement
    fn print_expression_statement(&mut self, expression: &Expression) -> GeneratorResult<()> {
        let needs_wrapping = matches!(
            expression,
            Expression::ObjectExpression { .. } | Expression::FunctionExpression(_)
        );

        if needs_wrapping {
            self.write("(")?;
        }

        self.print_expression(expression, Precedence::Sequence)?;

        if needs_wrapping {
            self.write(")")?;
        }

        self.print_semicolon_if_needed()?;
        Ok(())
    }

    /// Print a block statement
    fn print_block_statement(&mut self, body: &[Statement]) -> GeneratorResult<()> {
        self.print_block_statement_body(body)
    }

    /// Print block statement body with braces
    fn print_block_statement_body(&mut self, body: &[Statement]) -> GeneratorResult<()> {
        self.write("{")?;
        
        if !body.is_empty() {
            self.print_newline_if_needed()?;
            self.indent_level += 1;
            
            for stmt in body {
                self.print_indent_if_needed()?;
                self.print_statement(stmt)?;
                self.print_newline_if_needed()?;
            }
            
            self.indent_level -= 1;
            self.print_indent_if_needed()?;
        }
        
        self.write("}")?;
        Ok(())
    }

    /// Print a return statement
    fn print_return_statement(&mut self, argument: &Option<Expression>) -> GeneratorResult<()> {
        self.write("return")?;
        self.prev_token = Some(TokenType::Identifier);

        if let Some(arg) = argument {
            self.print_space_if_needed()?;
            self.print_expression(arg, Precedence::Sequence)?;
        }

        self.print_semicolon_if_needed()?;
        Ok(())
    }

    /// Print an expression with precedence context
    fn print_expression(&mut self, expr: &Expression, parent_precedence: Precedence) -> GeneratorResult<()> {
        match expr {
            Expression::Identifier(id) => self.print_identifier(id),
            Expression::Literal(lit) => self.print_literal(lit),
            Expression::BinaryExpression { left, operator, right } => {
                self.print_binary_expression(left, operator, right, parent_precedence)
            }
            Expression::TemplateLiteral { quasis, expressions } => {
                self.print_template_literal(quasis, expressions)
            }
            Expression::ThisExpression => self.print_this_expression(),
            _ => {
                // TODO: Implement remaining expression types
                self.write("/* EXPR */")?;
                Ok(())
            }
        }
    }

    /// Print a binary expression with precedence handling
    fn print_binary_expression(
        &mut self,
        left: &Expression,
        operator: &BinaryOperator,
        right: &Expression,
        parent_precedence: Precedence,
    ) -> GeneratorResult<()> {
        let precedence = self.get_binary_operator_precedence(operator);
        let needs_parens = precedence < parent_precedence;

        if needs_parens {
            self.write("(")?;
        }

        self.print_expression(left, precedence)?;
        self.print_binary_operator(operator)?;
        self.print_expression(right, precedence)?;

        if needs_parens {
            self.write(")")?;
        }

        Ok(())
    }

    /// Print binary operator
    fn print_binary_operator(&mut self, op: &BinaryOperator) -> GeneratorResult<()> {
        let op_str = match op {
            BinaryOperator::Add => "+",
            BinaryOperator::Subtract => "-",
            BinaryOperator::Multiply => "*",
            BinaryOperator::Divide => "/",
            BinaryOperator::Remainder => "%",
            BinaryOperator::Exponentiation => "**",
            BinaryOperator::Equal => "==",
            BinaryOperator::NotEqual => "!=",
            BinaryOperator::StrictEqual => "===",
            BinaryOperator::StrictNotEqual => "!==",
            BinaryOperator::LessThan => "<",
            BinaryOperator::LessThanEqual => "<=",
            BinaryOperator::GreaterThan => ">",
            BinaryOperator::GreaterThanEqual => ">=",
            BinaryOperator::LogicalAnd => "&&",
            BinaryOperator::LogicalOr => "||",
            _ => "/* OP */",
        };

        match self.config.format {
            crate::generator::OutputFormat::Compact => self.write(op_str),
            _ => {
                self.write(" ")?;
                self.write(op_str)?;
                self.write(" ")?;
                Ok(())
            }
        }
    }

    /// Get binary operator precedence
    fn get_binary_operator_precedence(&self, op: &BinaryOperator) -> Precedence {
        match op {
            BinaryOperator::Add | BinaryOperator::Subtract => Precedence::Additive,
            BinaryOperator::Multiply | BinaryOperator::Divide | BinaryOperator::Remainder => {
                Precedence::Multiplicative
            }
            BinaryOperator::Exponentiation => Precedence::Exponentiation,
            BinaryOperator::Equal
            | BinaryOperator::NotEqual
            | BinaryOperator::StrictEqual
            | BinaryOperator::StrictNotEqual => Precedence::Equality,
            BinaryOperator::LogicalAnd => Precedence::LogicalAnd,
            BinaryOperator::LogicalOr => Precedence::LogicalOr,
            _ => Precedence::Equality,
        }
    }

    /// Helper methods

    fn print_identifier(&mut self, id: &Identifier) -> GeneratorResult<()> {
        self.write(&id.name)?;
        self.prev_token = Some(TokenType::Identifier);
        Ok(())
    }

    fn print_literal(&mut self, lit: &Literal) -> GeneratorResult<()> {
        match lit {
            Literal::String(s) => self.print_string_literal(s),
            Literal::Number(n) => self.print_number_literal(n),
            Literal::Boolean(b) => self.print_boolean_literal(b),
            Literal::Null => self.print_null_literal(),
            Literal::RegExp(r) => self.print_regexp_literal(r),
        }
    }

    fn print_string_literal(&mut self, lit: &StringLiteral) -> GeneratorResult<()> {
        let quote_char = self.choose_quote_character(&lit.value);
        let escaped = self.escape_string(&lit.value, quote_char);
        
        self.write(&format!("{}{}{}", quote_char, escaped, quote_char))?;
        self.prev_token = Some(TokenType::String);
        Ok(())
    }

    fn print_number_literal(&mut self, lit: &NumberLiteral) -> GeneratorResult<()> {
        let canonical = self.canonicalize_number(lit.value);
        self.write(&canonical)?;
        self.prev_token = Some(TokenType::Number);
        Ok(())
    }

    fn print_boolean_literal(&mut self, lit: &BooleanLiteral) -> GeneratorResult<()> {
        self.write(if lit.value { "true" } else { "false" })?;
        self.prev_token = Some(TokenType::Identifier);
        Ok(())
    }

    fn print_null_literal(&mut self) -> GeneratorResult<()> {
        self.write("null")?;
        self.prev_token = Some(TokenType::Identifier);
        Ok(())
    }

    fn print_regexp_literal(&mut self, lit: &RegExpLiteral) -> GeneratorResult<()> {
        self.write(&format!("/{}/{}", lit.pattern, lit.flags))?;
        self.prev_token = Some(TokenType::Regex);
        Ok(())
    }

    fn print_this_expression(&mut self) -> GeneratorResult<()> {
        self.write("this")?;
        self.prev_token = Some(TokenType::Identifier);
        Ok(())
    }

    fn print_pattern(&mut self, pattern: &Pattern) -> GeneratorResult<()> {
        match pattern {
            Pattern::Identifier(id) => self.print_identifier(id),
            _ => self.write("/* PATTERN */"),
        }
    }

    fn print_parameter_list(&mut self, params: &[Pattern]) -> GeneratorResult<()> {
        for (i, param) in params.iter().enumerate() {
            if i > 0 {
                self.write(",")?;
                self.print_space_if_needed()?;
            }
            self.print_pattern(param)?;
        }
        Ok(())
    }

    /// Print a template literal expression
    fn print_template_literal(
        &mut self,
        quasis: &[crate::parser::ast_types::TemplateElement],
        expressions: &[Expression],
    ) -> GeneratorResult<()> {
        self.write("`")?;
        
        for (i, quasi) in quasis.iter().enumerate() {
            // Print the template element (escape backticks and ${ sequences)
            let escaped = self.escape_template_element(&quasi.value);
            self.write(&escaped)?;
            
            // If this is not the tail element, print the expression
            if !quasi.tail && i < expressions.len() {
                self.write("${")?;
                self.print_expression(&expressions[i], Precedence::Sequence)?;
                self.write("}")?;
            }
        }
        
        self.write("`")?;
        Ok(())
    }

    /// Utility methods

    fn write(&mut self, s: &str) -> GeneratorResult<()> {
        self.output.push_str(s);
        self.chars_written += s.len();
        Ok(())
    }

    fn print_space_if_needed(&mut self) -> GeneratorResult<()> {
        match self.config.format {
            crate::generator::OutputFormat::Compact => {
                // In compact mode, add space only when absolutely necessary for parsing
                // Always add space after keywords to avoid token fusion
                self.write(" ")?;
            }
            crate::generator::OutputFormat::Readable | crate::generator::OutputFormat::Pretty => {
                self.write(" ")?;
            }
        }
        Ok(())
    }

    fn print_newline_if_needed(&mut self) -> GeneratorResult<()> {
        match self.config.format {
            crate::generator::OutputFormat::Compact => {
                // No newlines in compact mode
            }
            crate::generator::OutputFormat::Readable | crate::generator::OutputFormat::Pretty => {
                match self.config.newline {
                    crate::generator::NewlineStyle::Lf => self.write("\n")?,
                    crate::generator::NewlineStyle::Crlf => self.write("\r\n")?,
                }
            }
        }
        Ok(())
    }

    fn print_indent_if_needed(&mut self) -> GeneratorResult<()> {
        match self.config.format {
            crate::generator::OutputFormat::Pretty => {
                let indent = "  ".repeat(self.indent_level);
                self.write(&indent)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn print_semicolon_if_needed(&mut self) -> GeneratorResult<()> {
        match self.config.semicolon {
            crate::generator::SemicolonStrategy::Always => self.write(";"),
            crate::generator::SemicolonStrategy::Auto => {
                if self.needs_semicolon_for_asi() {
                    self.write(";")
                } else {
                    Ok(())
                }
            }
            crate::generator::SemicolonStrategy::Remove => Ok(()),
        }
    }

    fn print_assignment_operator(&mut self) -> GeneratorResult<()> {
        match self.config.format {
            crate::generator::OutputFormat::Compact => self.write("="),
            _ => {
                self.write(" = ")?;
                Ok(())
            }
        }
    }

    fn print_statement_separator(&mut self) -> GeneratorResult<()> {
        match self.config.format {
            crate::generator::OutputFormat::Compact => {
                // In compact mode, statements are naturally separated by semicolons
                // No additional separator needed
            }
            _ => {
                self.print_newline_if_needed()?;
            }
        }
        Ok(())
    }

    /// String and number processing helpers

    fn choose_quote_character(&self, content: &str) -> char {
        match self.config.quote {
            crate::generator::QuoteStrategy::Single => '\'',
            crate::generator::QuoteStrategy::Double => '"',
            crate::generator::QuoteStrategy::Auto => {
                let single_count = content.chars().filter(|&c| c == '\'').count();
                let double_count = content.chars().filter(|&c| c == '"').count();
                if single_count <= double_count { '\'' } else { '"' }
            }
        }
    }

    fn escape_string(&self, content: &str, quote_char: char) -> String {
        let mut result = String::new();
        for ch in content.chars() {
            match ch {
                '\n' => result.push_str("\\n"),
                '\r' => result.push_str("\\r"),
                '\t' => result.push_str("\\t"),
                '\\' => result.push_str("\\\\"),
                c if c == quote_char => {
                    result.push('\\');
                    result.push(c);
                }
                c => result.push(c),
            }
        }
        result
    }

    fn escape_template_element(&self, content: &str) -> String {
        let mut result = String::new();
        let mut chars = content.chars().peekable();
        
        while let Some(ch) = chars.next() {
            match ch {
                '`' => result.push_str("\\`"),
                '\\' => result.push_str("\\\\"),
                '$' => {
                    // Check if this is the start of ${ sequence
                    if chars.peek() == Some(&'{') {
                        result.push_str("\\${");
                        chars.next(); // consume the '{'
                    } else {
                        result.push('$');
                    }
                }
                c => result.push(c),
            }
        }
        result
    }

    fn canonicalize_number(&self, value: f64) -> String {
        if value == value.trunc() && value.abs() < 1e15 {
            format!("{}", value as i64)
        } else {
            format!("{}", value)
        }
    }

    /// ASI and separation detection

    fn needs_space_for_separation(&self) -> bool {
        // Always need space after identifiers, numbers, and certain operators
        match self.prev_token {
            Some(TokenType::Identifier) => true,
            Some(TokenType::Number) => true,
            _ => false,
        }
    }

    fn needs_semicolon_for_asi(&self) -> bool {
        // For safety, always emit semicolons in auto mode for now
        // TODO: Implement proper ASI hazard detection
        true
    }

    /// Validation and error handling methods
    
    /// Validate program structure for common errors
    fn validate_program(&self, program: &Program) -> GeneratorResult<()> {
        // Check for empty program
        if program.body.is_empty() {
            return Ok(()); // Empty programs are valid
        }

        // Validate each statement
        for (i, stmt) in program.body.iter().enumerate() {
            self.validate_statement(stmt).map_err(|e| {
                use crate::generator::GeneratorError;
                match e {
                    GeneratorError::MalformedAst { message, node_type } => {
                        GeneratorError::MalformedAst {
                            message: format!("Statement {}: {}", i, message),
                            node_type,
                        }
                    }
                    _ => e,
                }
            })?;
        }

        Ok(())
    }

    /// Validate individual statement structure
    fn validate_statement(&self, stmt: &Statement) -> GeneratorResult<()> {
        use crate::generator::GeneratorError;
        
        match stmt {
            Statement::VariableDeclaration { declarations, kind: _ } => {
                if declarations.is_empty() {
                    return Err(GeneratorError::MissingRequiredField {
                        field: "declarations".to_string(),
                        node_type: "VariableDeclaration".to_string(),
                    });
                }
                
                for declarator in declarations {
                    self.validate_variable_declarator(declarator)?;
                }
            }
            Statement::FunctionDeclaration { id, params, body, .. } => {
                if let Some(id) = id {
                    self.validate_identifier(id)?;
                }
                
                for param in params {
                    self.validate_pattern(param)?;
                }
                
                self.validate_block_statement(body)?;
            }
            Statement::ExpressionStatement { expression } => {
                self.validate_expression(expression)?;
            }
            Statement::BlockStatement { body } => {
                for stmt in body {
                    self.validate_statement(stmt)?;
                }
            }
            Statement::ReturnStatement { argument } => {
                if let Some(expr) = argument {
                    self.validate_expression(expr)?;
                }
            }
            _ => {
                // For unimplemented statement types, just warn
                // This is not an error in the framework implementation
            }
        }
        
        Ok(())
    }

    /// Validate variable declarator
    fn validate_variable_declarator(&self, declarator: &VariableDeclarator) -> GeneratorResult<()> {
        self.validate_pattern(&declarator.id)?;
        
        if let Some(init) = &declarator.init {
            self.validate_expression(init)?;
        }
        
        Ok(())
    }

    /// Validate block statement
    fn validate_block_statement(&self, block: &BlockStatement) -> GeneratorResult<()> {
        for stmt in &block.body {
            self.validate_statement(stmt)?;
        }
        Ok(())
    }

    /// Validate expression structure
    fn validate_expression(&self, expr: &Expression) -> GeneratorResult<()> {
        use crate::generator::GeneratorError;
        
        match expr {
            Expression::Identifier(id) => {
                self.validate_identifier(id)?;
            }
            Expression::Literal(lit) => {
                self.validate_literal(lit)?;
            }
            Expression::BinaryExpression { left, operator: _, right } => {
                self.validate_expression(left)?;
                self.validate_expression(right)?;
            }
            Expression::TemplateLiteral { quasis, expressions } => {
                if quasis.is_empty() {
                    return Err(GeneratorError::MissingRequiredField {
                        field: "quasis".to_string(),
                        node_type: "TemplateLiteral".to_string(),
                    });
                }
                
                // Validate template literal structure
                if expressions.len() != quasis.len() - 1 && !(expressions.is_empty() && quasis.len() == 1) {
                    return Err(GeneratorError::TemplateLiteralError {
                        message: "Invalid template literal structure: quasis and expressions count mismatch".to_string(),
                        template: format!("quasis: {}, expressions: {}", quasis.len(), expressions.len()),
                    });
                }
                
                for expr in expressions {
                    self.validate_expression(expr)?;
                }
            }
            _ => {
                // For unimplemented expression types, this is not an error in framework
            }
        }
        
        Ok(())
    }

    /// Validate identifier
    fn validate_identifier(&self, id: &Identifier) -> GeneratorResult<()> {
        use crate::generator::GeneratorError;
        
        if id.name.is_empty() {
            return Err(GeneratorError::IdentifierError {
                message: "Identifier name cannot be empty".to_string(),
                identifier: "<empty>".to_string(),
            });
        }
        
        // Check for invalid identifier characters (basic validation)
        if !id.name.chars().next().unwrap_or('_').is_alphabetic() && id.name.chars().next() != Some('_') && id.name.chars().next() != Some('$') {
            return Err(GeneratorError::IdentifierError {
                message: "Identifier must start with letter, underscore, or dollar sign".to_string(),
                identifier: id.name.clone(),
            });
        }
        
        Ok(())
    }

    /// Validate literal values
    fn validate_literal(&self, lit: &Literal) -> GeneratorResult<()> {
        use crate::generator::GeneratorError;
        
        match lit {
            Literal::String(s) => {
                // Basic string validation - check for proper content
                if s.value.contains('\0') {
                    return Err(GeneratorError::StringProcessingError {
                        message: "String contains null character".to_string(),
                        content: s.value.clone(),
                    });
                }
            }
            Literal::Number(n) => {
                // Validate numeric values
                if n.value.is_nan() {
                    return Err(GeneratorError::NumericValueError {
                        message: "NaN values are not supported in compact mode".to_string(),
                        value: "NaN".to_string(),
                    });
                }
                
                if n.value.is_infinite() {
                    return Err(GeneratorError::NumericValueError {
                        message: "Infinite values require careful handling".to_string(),
                        value: n.value.to_string(),
                    });
                }
            }
            Literal::RegExp(r) => {
                // Basic regex validation
                if r.pattern.is_empty() {
                    return Err(GeneratorError::StringProcessingError {
                        message: "RegExp pattern cannot be empty".to_string(),
                        content: r.pattern.clone(),
                    });
                }
            }
            _ => {}
        }
        
        Ok(())
    }

    /// Validate pattern structure
    fn validate_pattern(&self, pattern: &Pattern) -> GeneratorResult<()> {
        match pattern {
            Pattern::Identifier(id) => {
                self.validate_identifier(id)?;
            }
            _ => {
                // Other pattern types are not implemented yet
            }
        }
        
        Ok(())
    }

    /// Check for potential ASI hazards
    fn check_asi_hazard(&self, _stmt: &Statement) -> GeneratorResult<()> {
        // TODO: Implement ASI hazard detection
        // This would check for patterns like:
        // - return\nvalue
        // - ++\nvar
        // - continue\nlabel
        // etc.
        Ok(())
    }

    /// Validate memory usage during generation
    fn check_memory_limits(&self) -> GeneratorResult<()> {
        use crate::generator::GeneratorError;
        
        const MAX_OUTPUT_SIZE: usize = 10 * 1024 * 1024; // 10MB limit
        
        if self.output.len() > MAX_OUTPUT_SIZE {
            return Err(GeneratorError::OutputSizeLimitExceeded {
                current_size: self.output.len(),
                limit: MAX_OUTPUT_SIZE,
            });
        }
        
        Ok(())
    }

    /// Performance optimization methods
    
    /// Pre-populate indent cache for performance
    fn populate_indent_cache(&mut self) {
        const MAX_CACHED_INDENT: usize = 20;
        self.indent_cache.reserve(MAX_CACHED_INDENT);
        
        for i in 0..MAX_CACHED_INDENT {
            self.indent_cache.push("  ".repeat(i));
        }
    }

    /// Optimize output capacity based on format
    fn optimize_output_capacity(&mut self) {
        let estimated_capacity = match self.config.format {
            crate::generator::OutputFormat::Compact => 2048,     // Compact output
            crate::generator::OutputFormat::Readable => 4096,    // Readable with some formatting
            crate::generator::OutputFormat::Pretty => 8192,     // Pretty with full formatting
        };
        
        self.output.reserve(estimated_capacity);
    }

    /// Get performance metrics
    pub fn get_performance_metrics(&self) -> PerformanceMetrics {
        PerformanceMetrics {
            chars_written: self.chars_written,
            output_capacity: self.output.capacity(),
            output_length: self.output.len(),
            cache_utilization: if self.indent_cache.is_empty() { 0.0 } else { 
                (self.indent_level as f64) / (self.indent_cache.len() as f64) 
            },
        }
    }

    /// Reset printer state for reuse
    pub fn reset(&mut self) {
        self.output.clear();
        self.warnings.clear();
        self.prev_token = None;
        self.indent_level = 0;
        self.chars_written = 0;
        self.string_buffer.clear();
        // Keep the caches for reuse
    }
}

/// Performance metrics for monitoring
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub chars_written: usize,
    pub output_capacity: usize,
    pub output_length: usize,
    pub cache_utilization: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::GeneratorConfig;

    #[test]
    fn test_printer_creation() {
        let config = GeneratorConfig::default();
        let printer = Printer::new(&config);
        assert!(printer.output.is_empty());
    }

    #[test]
    fn test_empty_program() {
        let config = GeneratorConfig::default();
        let mut printer = Printer::new(&config);
        let program = Program {
            body: vec![],
            source_type: ProgramSourceType::Script,
        };
        
        let result = printer.print_program(&program).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_simple_variable_declaration() {
        let config = GeneratorConfig::default();
        let mut printer = Printer::new(&config);
        
        let program = Program {
            body: vec![Statement::VariableDeclaration {
                declarations: vec![VariableDeclarator {
                    id: Pattern::Identifier(Identifier { name: "x".to_string() }),
                    init: Some(Expression::Literal(Literal::Number(NumberLiteral { value: 5.0 }))),
                }],
                kind: VariableDeclarationKind::Let,
            }],
            source_type: ProgramSourceType::Script,
        };
        
        let result = printer.print_program(&program).unwrap();
        assert_eq!(result, "let x=5;");
    }
}