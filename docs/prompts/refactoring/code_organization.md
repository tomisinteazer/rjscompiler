# Rust Code Refactoring and Organization Standards

## Code Organization Principles

### Module Structure
Break down monolithic functions into smaller, focused functions with clear responsibilities:

```rust
// Good: Well-organized module structure
pub mod parser {
    pub mod lexer;
    pub mod ast;
    pub mod error;
    
    pub use lexer::Lexer;
    pub use ast::{AstNode, NodeKind};
    pub use error::ParseError;
}

pub mod analyzer {
    pub mod scope;
    pub mod symbols;
    pub mod types;
    
    pub use scope::{Scope, ScopeAnalyzer};
    pub use symbols::{Symbol, SymbolTable};
}

// Good: Clear separation of concerns
impl MinifierEngine {
    /// High-level orchestration function
    pub fn minify(&mut self, source: &str) -> Result<String, MinifierError> {
        let ast = self.parse_source(source)?;
        let analyzed_ast = self.analyze_ast(ast)?;
        let transformed_ast = self.transform_ast(analyzed_ast)?;
        let output = self.generate_code(transformed_ast)?;
        Ok(output)
    }
    
    /// Focused parsing responsibility
    fn parse_source(&self, source: &str) -> Result<AstNode, ParseError> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        parser.parse()
    }
    
    /// Focused analysis responsibility
    fn analyze_ast(&self, ast: AstNode) -> Result<AnalyzedAst, AnalysisError> {
        let mut analyzer = ScopeAnalyzer::new();
        analyzer.analyze(ast)
    }
}
```

### Trait-Based Design
Use traits to define clear interfaces and enable extensibility:

```rust
// Good: Clear trait interface for transformations
pub trait AstTransformer {
    type Error;
    
    fn transform(&mut self, ast: AstNode) -> Result<AstNode, Self::Error>;
    fn can_transform(&self, node: &AstNode) -> bool;
}

// Good: Implement specific transformers
pub struct VariableRenamer {
    symbol_table: SymbolTable,
    naming_strategy: NamingStrategy,
}

impl AstTransformer for VariableRenamer {
    type Error = RenameError;
    
    fn transform(&mut self, mut ast: AstNode) -> Result<AstNode, Self::Error> {
        if self.can_transform(&ast) {
            self.rename_variables_in_node(&mut ast)?;
        }
        Ok(ast)
    }
    
    fn can_transform(&self, node: &AstNode) -> bool {
        matches!(node.kind(), NodeKind::Variable | NodeKind::Function)
    }
}

// Good: Composable transformation pipeline
pub struct TransformationPipeline {
    transformers: Vec<Box<dyn AstTransformer<Error = TransformError>>>,
}

impl TransformationPipeline {
    pub fn new() -> Self {
        Self {
            transformers: Vec::new(),
        }
    }
    
    pub fn add_transformer<T>(&mut self, transformer: T) 
    where 
        T: AstTransformer<Error = TransformError> + 'static,
    {
        self.transformers.push(Box::new(transformer));
    }
    
    pub fn apply_all(&mut self, ast: AstNode) -> Result<AstNode, TransformError> {
        self.transformers.iter_mut().try_fold(ast, |acc, transformer| {
            transformer.transform(acc)
        })
    }
}
```

## Function Decomposition

### Single Responsibility Principle
Each function should have one clear purpose:

```rust
// Good: Functions with single, clear responsibilities
impl Parser {
    /// Parse a complete JavaScript function
    pub fn parse_function(&mut self) -> Result<FunctionNode, ParseError> {
        self.expect_keyword("function")?;
        let name = self.parse_identifier()?;
        let params = self.parse_parameter_list()?;
        let body = self.parse_block_statement()?;
        
        Ok(FunctionNode::new(name, params, body))
    }
    
    /// Parse function parameter list
    fn parse_parameter_list(&mut self) -> Result<Vec<Parameter>, ParseError> {
        self.expect_token(TokenKind::LeftParen)?;
        
        let mut params = Vec::new();
        
        if !self.check_token(TokenKind::RightParen) {
            loop {
                params.push(self.parse_parameter()?);
                
                if !self.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }
        
        self.expect_token(TokenKind::RightParen)?;
        Ok(params)
    }
    
    /// Parse a single parameter
    fn parse_parameter(&mut self) -> Result<Parameter, ParseError> {
        let name = self.parse_identifier()?;
        let default_value = if self.match_token(TokenKind::Equals) {
            Some(self.parse_expression()?)
        } else {
            None
        };
        
        Ok(Parameter::new(name, default_value))
    }
}

// Avoid: Monolithic function doing too much
impl Parser {
    // BAD: This function has too many responsibilities
    pub fn parse_function_bad(&mut self) -> Result<FunctionNode, ParseError> {
        // Parsing function keyword
        if !matches!(self.current_token(), Some(Token { kind: TokenKind::Keyword, text }) if text == "function") {
            return Err(ParseError::ExpectedKeyword("function".to_string()));
        }
        self.advance();
        
        // Parsing identifier
        let name = match self.current_token() {
            Some(Token { kind: TokenKind::Identifier, text }) => {
                let name = text.clone();
                self.advance();
                name
            }
            _ => return Err(ParseError::ExpectedIdentifier),
        };
        
        // Parsing parameters (inline, making function too long)
        if !matches!(self.current_token(), Some(Token { kind: TokenKind::LeftParen, .. })) {
            return Err(ParseError::ExpectedToken(TokenKind::LeftParen));
        }
        self.advance();
        
        let mut params = Vec::new();
        // ... lots more inline parsing logic
        
        // This function is doing too much!
    }
}
```

### Error Handling Decomposition
Separate error handling concerns:

```rust
// Good: Specific error types for different concerns
#[derive(Debug, thiserror::Error)]
pub enum MinifierError {
    #[error("Parse error")]
    Parse(#[from] ParseError),
    
    #[error("Analysis error")]
    Analysis(#[from] AnalysisError),
    
    #[error("Transformation error")]
    Transform(#[from] TransformError),
    
    #[error("Generation error")]
    Generation(#[from] GenerationError),
    
    #[error("IO error")]
    Io(#[from] std::io::Error),
}

// Good: Error context preservation
impl MinifierEngine {
    fn process_file(&self, path: &Path) -> Result<String, MinifierError> {
        let source = std::fs::read_to_string(path)
            .map_err(MinifierError::Io)?;
        
        self.minify(&source)
            .map_err(|e| {
                log::error!("Failed to minify file {}: {}", path.display(), e);
                e
            })
    }
}

// Good: Error recovery strategies
impl Parser {
    fn parse_statement_list(&mut self) -> (Vec<Statement>, Vec<ParseError>) {
        let mut statements = Vec::new();
        let mut errors = Vec::new();
        
        while !self.is_at_end() {
            match self.parse_statement() {
                Ok(stmt) => statements.push(stmt),
                Err(e) => {
                    errors.push(e);
                    self.synchronize(); // Recover to next statement boundary
                }
            }
        }
        
        (statements, errors)
    }
    
    fn synchronize(&mut self) {
        self.advance();
        
        while !self.is_at_end() {
            if self.previous_token_is(TokenKind::Semicolon) {
                return;
            }
            
            if self.current_token_is_statement_start() {
                return;
            }
            
            self.advance();
        }
    }
}
```

## Data Structure Organization

### Type-Driven Design
Use the type system to enforce correctness:

```rust
// Good: Use newtypes to prevent confusion
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VariableId(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScopeId(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FunctionId(usize);

// Good: Prevent mixing up different ID types
impl SymbolTable {
    pub fn get_variable(&self, id: VariableId) -> Option<&Variable> {
        self.variables.get(id.0)
    }
    
    pub fn get_scope(&self, id: ScopeId) -> Option<&Scope> {
        self.scopes.get(id.0)
    }
    
    // Compiler prevents passing wrong ID type
}

// Good: Use builder pattern for complex construction
pub struct MinifierConfigBuilder {
    optimization_level: OptimizationLevel,
    preserve_comments: bool,
    source_maps: bool,
    target_version: Option<JavaScriptVersion>,
}

impl MinifierConfigBuilder {
    pub fn new() -> Self {
        Self {
            optimization_level: OptimizationLevel::Safe,
            preserve_comments: false,
            source_maps: false,
            target_version: None,
        }
    }
    
    pub fn optimization_level(mut self, level: OptimizationLevel) -> Self {
        self.optimization_level = level;
        self
    }
    
    pub fn preserve_comments(mut self, preserve: bool) -> Self {
        self.preserve_comments = preserve;
        self
    }
    
    pub fn build(self) -> Result<MinifierConfig, ConfigError> {
        let target_version = self.target_version
            .unwrap_or(JavaScriptVersion::Es2018);
        
        Ok(MinifierConfig {
            optimization_level: self.optimization_level,
            preserve_comments: self.preserve_comments,
            source_maps: self.source_maps,
            target_version,
        })
    }
}

// Usage
let config = MinifierConfigBuilder::new()
    .optimization_level(OptimizationLevel::Aggressive)
    .preserve_comments(true)
    .build()?;
```

### Visitor Pattern Implementation
Implement traversal patterns cleanly:

```rust
// Good: Generic visitor trait
pub trait AstVisitor {
    fn visit_function(&mut self, func: &FunctionNode) -> VisitResult {
        walk_function(self, func)
    }
    
    fn visit_variable(&mut self, var: &VariableNode) -> VisitResult {
        walk_variable(self, var)
    }
    
    fn visit_expression(&mut self, expr: &ExpressionNode) -> VisitResult {
        walk_expression(self, expr)
    }
}

// Good: Default walking implementation
pub fn walk_function<V: AstVisitor>(visitor: &mut V, func: &FunctionNode) -> VisitResult {
    for param in &func.parameters {
        visitor.visit_variable(param)?;
    }
    
    visitor.visit_block(&func.body)
}

// Good: Specific visitor implementations
pub struct VariableCollector {
    variables: Vec<VariableId>,
}

impl AstVisitor for VariableCollector {
    fn visit_variable(&mut self, var: &VariableNode) -> VisitResult {
        self.variables.push(var.id());
        Ok(())
    }
}

pub struct ScopeAnalyzer {
    current_scope: ScopeId,
    scope_stack: Vec<ScopeId>,
    symbol_table: SymbolTable,
}

impl AstVisitor for ScopeAnalyzer {
    fn visit_function(&mut self, func: &FunctionNode) -> VisitResult {
        let function_scope = self.symbol_table.create_scope(Some(self.current_scope));
        self.scope_stack.push(self.current_scope);
        self.current_scope = function_scope;
        
        // Add parameters to function scope
        for param in &func.parameters {
            self.symbol_table.add_symbol(
                function_scope,
                param.name().clone(),
                SymbolType::Parameter,
            );
        }
        
        // Visit function body
        let result = walk_function(self, func);
        
        // Restore previous scope
        self.current_scope = self.scope_stack.pop()
            .expect("Scope stack should not be empty");
        
        result
    }
}
```

## Configuration and Dependency Injection

### Flexible Configuration
Design configuration systems that are easy to extend:

```rust
// Good: Layered configuration system
#[derive(Debug, Clone)]
pub struct MinifierConfig {
    pub transformations: TransformationConfig,
    pub output: OutputConfig,
    pub debug: DebugConfig,
}

#[derive(Debug, Clone)]
pub struct TransformationConfig {
    pub rename_variables: bool,
    pub remove_dead_code: bool,
    pub inline_functions: bool,
    pub constant_folding: bool,
}

#[derive(Debug, Clone)]
pub struct OutputConfig {
    pub source_maps: bool,
    pub preserve_comments: Vec<CommentType>,
    pub target_version: JavaScriptVersion,
}

// Good: Configuration merging
impl MinifierConfig {
    pub fn merge_with_file(mut self, path: &Path) -> Result<Self, ConfigError> {
        let file_config = Self::load_from_file(path)?;
        self.merge(file_config);
        Ok(self)
    }
    
    pub fn merge(&mut self, other: Self) {
        self.transformations.merge(other.transformations);
        self.output.merge(other.output);
        self.debug.merge(other.debug);
    }
}

// Good: Dependency injection for testability
pub struct MinifierEngine<P, A, T, G> {
    parser: P,
    analyzer: A,
    transformer: T,
    generator: G,
}

impl<P, A, T, G> MinifierEngine<P, A, T, G>
where
    P: Parser,
    A: Analyzer,
    T: Transformer,
    G: Generator,
{
    pub fn new(parser: P, analyzer: A, transformer: T, generator: G) -> Self {
        Self {
            parser,
            analyzer,
            transformer,
            generator,
        }
    }
    
    pub fn minify(&mut self, source: &str) -> Result<String, MinifierError> {
        let ast = self.parser.parse(source)?;
        let analyzed = self.analyzer.analyze(ast)?;
        let transformed = self.transformer.transform(analyzed)?;
        let output = self.generator.generate(transformed)?;
        Ok(output)
    }
}
```

## Testing Organization

### Test Structure
Organize tests for maintainability:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    mod parser_tests {
        use super::*;
        
        #[test]
        fn should_parse_simple_function() {
            let source = "function test() { return 42; }";
            let ast = parse_javascript(source).unwrap();
            
            assert!(matches!(ast.kind(), NodeKind::Function));
            assert_eq!(ast.as_function().unwrap().name(), "test");
        }
        
        #[test]
        fn should_handle_malformed_function() {
            let source = "function test( { return 42; }";
            let result = parse_javascript(source);
            
            assert!(result.is_err());
        }
    }
    
    mod transformation_tests {
        use super::*;
        
        #[test]
        fn should_rename_variables_safely() {
            let source = r#"
                function outer() {
                    var x = 1;
                    function inner() {
                        var y = 2;
                        return x + y;
                    }
                    return inner();
                }
            "#;
            
            let result = minify_javascript(source).unwrap();
            // Verify that variable relationships are preserved
            assert!(verify_semantic_equivalence(source, &result));
        }
    }
    
    mod integration_tests {
        use super::*;
        
        #[test]
        fn full_minification_workflow() {
            let source = include_str!("../test_data/complex_example.js");
            let result = minify_javascript(source).unwrap();
            
            // Verify size reduction
            assert!(result.len() < source.len() * 0.7); // At least 30% reduction
            
            // Verify semantic equivalence
            assert!(verify_semantic_equivalence(source, &result));
        }
    }
}
```