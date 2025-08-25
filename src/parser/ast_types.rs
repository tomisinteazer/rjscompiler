//! # AST Types Module
//!
//! This module defines the Abstract Syntax Tree (AST) node types used by the parser.
//! It provides a simplified, serializable representation of the OXC AST that can be
//! easily processed by the minification engine.

use oxc_ast::ast as oxc;
use serde::{Deserialize, Serialize};

/// Root program node containing all statements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Program {
    /// Program body containing statements
    pub body: Vec<Statement>,
    /// Source type (script or module)
    pub source_type: ProgramSourceType,
}

/// Program source type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProgramSourceType {
    Script,
    Module,
}

/// JavaScript statements
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Statement {
    /// Variable declaration: let, const, var
    VariableDeclaration {
        declarations: Vec<VariableDeclarator>,
        kind: VariableDeclarationKind,
    },
    /// Function declaration
    FunctionDeclaration {
        id: Option<Identifier>,
        params: Vec<Pattern>,
        body: BlockStatement,
        is_async: bool,
        is_generator: bool,
    },
    /// Class declaration
    ClassDeclaration {
        id: Option<Identifier>,
        super_class: Option<Box<Expression>>,
        body: ClassBody,
    },
    /// Expression statement
    ExpressionStatement {
        expression: Expression,
    },
    /// Block statement
    BlockStatement {
        body: Vec<Statement>,
    },
    /// Return statement
    ReturnStatement {
        argument: Option<Expression>,
    },
    /// If statement
    IfStatement {
        test: Expression,
        consequent: Box<Statement>,
        alternate: Option<Box<Statement>>,
    },
    /// While loop
    WhileStatement {
        test: Expression,
        body: Box<Statement>,
    },
    /// For loop
    ForStatement {
        init: Option<ForInit>,
        test: Option<Expression>,
        update: Option<Expression>,
        body: Box<Statement>,
    },
    /// Import declaration (ES6 modules)
    ImportDeclaration {
        specifiers: Vec<ImportSpecifier>,
        source: StringLiteral,
    },
    /// Export declaration (ES6 modules)
    ExportNamedDeclaration {
        declaration: Option<Box<Statement>>,
        specifiers: Vec<ExportSpecifier>,
        source: Option<StringLiteral>,
    },
}

/// Variable declaration kind
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VariableDeclarationKind {
    Var,
    Let,
    Const,
}

/// Variable declarator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableDeclarator {
    pub id: Pattern,
    pub init: Option<Expression>,
}

/// Block statement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockStatement {
    pub body: Vec<Statement>,
}

/// Class body
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassBody {
    pub body: Vec<ClassElement>,
}

/// Class element (method, property, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClassElement {
    /// Property definition
    PropertyDefinition {
        key: PropertyKey,
        value: Option<Expression>,
        is_static: bool,
        is_private: bool,
    },
    /// Method definition
    MethodDefinition {
        key: PropertyKey,
        value: FunctionExpression,
        kind: MethodKind,
        is_static: bool,
        is_private: bool,
    },
}

/// Method kind
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MethodKind {
    Constructor,
    Method,
    Get,
    Set,
}

/// For loop initialization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ForInit {
    VariableDeclaration {
        declarations: Vec<VariableDeclarator>,
        kind: VariableDeclarationKind,
    },
    Expression(Expression),
}

/// Import specifier
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ImportSpecifier {
    ImportDefaultSpecifier {
        local: Identifier,
    },
    ImportNamespaceSpecifier {
        local: Identifier,
    },
    ImportSpecifier {
        imported: Identifier,
        local: Identifier,
    },
}

/// Export specifier
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ExportSpecifier {
    ExportSpecifier {
        local: Identifier,
        exported: Identifier,
    },
}

/// JavaScript expressions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Expression {
    /// Identifier
    Identifier(Identifier),
    /// Literal values
    Literal(Literal),
    /// Binary expression (a + b, a === b, etc.)
    BinaryExpression {
        left: Box<Expression>,
        operator: BinaryOperator,
        right: Box<Expression>,
    },
    /// Unary expression (!a, -a, etc.)
    UnaryExpression {
        operator: UnaryOperator,
        argument: Box<Expression>,
        prefix: bool,
    },
    /// Assignment expression (a = b, a += b, etc.)
    AssignmentExpression {
        left: Box<Expression>,
        operator: AssignmentOperator,
        right: Box<Expression>,
    },
    /// Update expression (++a, a--, etc.)
    UpdateExpression {
        operator: UpdateOperator,
        argument: Box<Expression>,
        prefix: bool,
    },
    /// Function call
    CallExpression {
        callee: Box<Expression>,
        arguments: Vec<Expression>,
    },
    /// Member access (a.b, a[b])
    MemberExpression {
        object: Box<Expression>,
        property: Box<Expression>,
        computed: bool,
    },
    /// Function expression
    FunctionExpression(FunctionExpression),
    /// Arrow function expression
    ArrowFunctionExpression {
        params: Vec<Pattern>,
        body: ArrowFunctionBody,
        is_async: bool,
    },
    /// Object expression
    ObjectExpression {
        properties: Vec<ObjectProperty>,
    },
    /// Array expression
    ArrayExpression {
        elements: Vec<Option<Expression>>,
    },
    /// Template literal
    TemplateLiteral {
        quasis: Vec<TemplateElement>,
        expressions: Vec<Expression>,
    },
    /// Conditional expression (a ? b : c)
    ConditionalExpression {
        test: Box<Expression>,
        consequent: Box<Expression>,
        alternate: Box<Expression>,
    },
}

/// Function expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionExpression {
    pub id: Option<Identifier>,
    pub params: Vec<Pattern>,
    pub body: BlockStatement,
    pub is_async: bool,
    pub is_generator: bool,
}

/// Arrow function body
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ArrowFunctionBody {
    BlockStatement(BlockStatement),
    Expression(Box<Expression>),
}

/// Object property
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ObjectProperty {
    Property {
        key: PropertyKey,
        value: Expression,
        kind: PropertyKind,
        method: bool,
        shorthand: bool,
        computed: bool,
    },
    SpreadElement {
        argument: Expression,
    },
}

/// Property kind
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PropertyKind {
    Init,
    Get,
    Set,
}

/// Property key
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PropertyKey {
    Identifier(Identifier),
    Literal(Literal),
    PrivateName(PrivateName),
}

/// Private name (#x)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateName {
    pub name: String,
}

/// Template element
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateElement {
    pub value: String,
    pub tail: bool,
}

/// Patterns (for destructuring, parameters, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Pattern {
    Identifier(Identifier),
    ArrayPattern {
        elements: Vec<Option<Pattern>>,
    },
    ObjectPattern {
        properties: Vec<ObjectPatternProperty>,
    },
    AssignmentPattern {
        left: Box<Pattern>,
        right: Expression,
    },
    RestElement {
        argument: Box<Pattern>,
    },
}

/// Object pattern property
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ObjectPatternProperty {
    Property {
        key: PropertyKey,
        value: Pattern,
        computed: bool,
        shorthand: bool,
    },
    RestElement {
        argument: Pattern,
    },
}

/// Identifier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identifier {
    pub name: String,
}

/// Literal values
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Literal {
    /// String literal
    String(StringLiteral),
    /// Number literal
    Number(NumberLiteral),
    /// Boolean literal
    Boolean(BooleanLiteral),
    /// Null literal
    Null,
    /// Regular expression literal
    RegExp(RegExpLiteral),
}

/// String literal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringLiteral {
    pub value: String,
}

/// Number literal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumberLiteral {
    pub value: f64,
}

/// Boolean literal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BooleanLiteral {
    pub value: bool,
}

/// Regular expression literal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegExpLiteral {
    pub pattern: String,
    pub flags: String,
}

/// Binary operators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
    Exponentiation,
    Equal,
    NotEqual,
    StrictEqual,
    StrictNotEqual,
    LessThan,
    LessThanEqual,
    GreaterThan,
    GreaterThanEqual,
    LeftShift,
    RightShift,
    UnsignedRightShift,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    LogicalAnd,
    LogicalOr,
    In,
    Instanceof,
}

/// Unary operators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UnaryOperator {
    Plus,
    Minus,
    LogicalNot,
    BitwiseNot,
    Typeof,
    Void,
    Delete,
}

/// Assignment operators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssignmentOperator {
    Assign,
    AddAssign,
    SubtractAssign,
    MultiplyAssign,
    DivideAssign,
    RemainderAssign,
    ExponentiationAssign,
    LeftShiftAssign,
    RightShiftAssign,
    UnsignedRightShiftAssign,
    BitwiseAndAssign,
    BitwiseOrAssign,
    BitwiseXorAssign,
    LogicalAndAssign,
    LogicalOrAssign,
    NullishCoalescingAssign,
}

/// Update operators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpdateOperator {
    Increment,
    Decrement,
}

impl Program {
    /// Convert from OXC Program to our Program type
    pub fn from_oxc(oxc_program: &oxc::Program<'_>) -> Self {
        let body = oxc_program
            .body
            .iter()
            .filter_map(|stmt| Statement::from_oxc(stmt))
            .collect();

        let source_type = if oxc_program.source_type.is_module() {
            ProgramSourceType::Module
        } else {
            ProgramSourceType::Script
        };

        Self { body, source_type }
    }
}

impl Statement {
    /// Convert from OXC Statement to our Statement type
    pub fn from_oxc(oxc_stmt: &oxc::Statement<'_>) -> Option<Self> {
        match oxc_stmt {
            oxc::Statement::VariableDeclaration(decl) => {
                let kind = match decl.kind {
                    oxc::VariableDeclarationKind::Var => VariableDeclarationKind::Var,
                    oxc::VariableDeclarationKind::Let => VariableDeclarationKind::Let,
                    oxc::VariableDeclarationKind::Const => VariableDeclarationKind::Const,
                    _ => return None,
                };

                let declarations = decl
                    .declarations
                    .iter()
                    .filter_map(|decl| VariableDeclarator::from_oxc(decl))
                    .collect();

                Some(Statement::VariableDeclaration { declarations, kind })
            }
            oxc::Statement::FunctionDeclaration(func) => {
                let id = func.id.as_ref().map(|id| Identifier {
                    name: id.name.to_string(),
                });
                
                let params = func.params.items.iter()
                    .filter_map(|param| Pattern::from_oxc(&param.pattern))
                    .collect();
                
                let body = BlockStatement {
                    body: func.body.as_ref()?
                        .statements.iter()
                        .filter_map(|stmt| Statement::from_oxc(stmt))
                        .collect(),
                };
                
                Some(Statement::FunctionDeclaration {
                    id,
                    params,
                    body,
                    is_async: func.r#async,
                    is_generator: func.generator,
                })
            }
            oxc::Statement::ClassDeclaration(class) => {
                let id = class.id.as_ref().map(|id| Identifier {
                    name: id.name.to_string(),
                });
                
                let super_class = class.super_class.as_ref()
                    .and_then(|expr| Expression::from_oxc(expr))
                    .map(Box::new);
                
                let body = ClassBody {
                    body: class.body.body.iter()
                        .filter_map(|elem| ClassElement::from_oxc(elem))
                        .collect(),
                };
                
                Some(Statement::ClassDeclaration {
                    id,
                    super_class,
                    body,
                })
            }
            oxc::Statement::ExpressionStatement(stmt) => {
                Expression::from_oxc(&stmt.expression).map(|expression| {
                    Statement::ExpressionStatement { expression }
                })
            }
            oxc::Statement::BlockStatement(block) => {
                let body = block.body.iter()
                    .filter_map(|stmt| Statement::from_oxc(stmt))
                    .collect();
                Some(Statement::BlockStatement { body })
            }
            oxc::Statement::ReturnStatement(stmt) => {
                let argument = stmt.argument.as_ref().and_then(|expr| Expression::from_oxc(expr));
                Some(Statement::ReturnStatement { argument })
            }
            oxc::Statement::IfStatement(if_stmt) => {
                let test = Expression::from_oxc(&if_stmt.test)?;
                let consequent = Box::new(Statement::from_oxc(&if_stmt.consequent)?);
                let alternate = if_stmt.alternate.as_ref()
                    .and_then(|stmt| Statement::from_oxc(stmt))
                    .map(Box::new);
                
                Some(Statement::IfStatement {
                    test,
                    consequent,
                    alternate,
                })
            }
            oxc::Statement::WhileStatement(while_stmt) => {
                let test = Expression::from_oxc(&while_stmt.test)?;
                let body = Box::new(Statement::from_oxc(&while_stmt.body)?);
                
                Some(Statement::WhileStatement { test, body })
            }
            oxc::Statement::ForStatement(for_stmt) => {
                let init = for_stmt.init.as_ref().and_then(|init| {
                    if let Some(expr) = init.as_expression() {
                        Expression::from_oxc(expr).map(ForInit::Expression)
                    } else {
                        // For now, skip variable declarations in for loops
                        None
                    }
                });
                
                let test = for_stmt.test.as_ref().and_then(|expr| Expression::from_oxc(expr));
                let update = for_stmt.update.as_ref().and_then(|expr| Expression::from_oxc(expr));
                let body = Box::new(Statement::from_oxc(&for_stmt.body)?);
                
                Some(Statement::ForStatement {
                    init,
                    test,
                    update,
                    body,
                })
            }
            // TODO: Add more statement types as needed
            _ => None,
        }
    }
}

impl VariableDeclarator {
    /// Convert from OXC VariableDeclarator to our VariableDeclarator type
    pub fn from_oxc(oxc_decl: &oxc::VariableDeclarator<'_>) -> Option<Self> {
        let id = Pattern::from_oxc(&oxc_decl.id)?;
        let init = oxc_decl.init.as_ref().and_then(|expr| Expression::from_oxc(expr));

        Some(Self { id, init })
    }
}

impl ClassElement {
    /// Convert from OXC ClassElement to our ClassElement type
    pub fn from_oxc(oxc_elem: &oxc::ClassElement<'_>) -> Option<Self> {
        match oxc_elem {
            oxc::ClassElement::PropertyDefinition(prop) => {
                let key = PropertyKey::from_oxc(&prop.key)?;
                let value = prop.value.as_ref().and_then(|expr| Expression::from_oxc(expr));
                let is_static = prop.r#static;
                let is_private = matches!(&prop.key, oxc::PropertyKey::PrivateIdentifier(_));
                
                Some(ClassElement::PropertyDefinition {
                    key,
                    value,
                    is_static,
                    is_private,
                })
            }
            oxc::ClassElement::MethodDefinition(method) => {
                let key = PropertyKey::from_oxc(&method.key)?;
                let value = FunctionExpression::from_oxc(&method.value)?;
                let kind = match method.kind {
                    oxc::MethodDefinitionKind::Constructor => MethodKind::Constructor,
                    oxc::MethodDefinitionKind::Method => MethodKind::Method,
                    oxc::MethodDefinitionKind::Get => MethodKind::Get,
                    oxc::MethodDefinitionKind::Set => MethodKind::Set,
                };
                let is_static = method.r#static;
                let is_private = matches!(&method.key, oxc::PropertyKey::PrivateIdentifier(_));
                
                Some(ClassElement::MethodDefinition {
                    key,
                    value,
                    kind,
                    is_static,
                    is_private,
                })
            }
            _ => None,
        }
    }
}

impl Expression {
    /// Convert from OXC Expression to our Expression type
    pub fn from_oxc(oxc_expr: &oxc::Expression<'_>) -> Option<Self> {
        match oxc_expr {
            oxc::Expression::Identifier(id) => {
                Some(Expression::Identifier(Identifier {
                    name: id.name.to_string(),
                }))
            }
            oxc::Expression::NumericLiteral(lit) => {
                Some(Expression::Literal(Literal::Number(NumberLiteral {
                    value: lit.value,
                })))
            }
            oxc::Expression::StringLiteral(lit) => {
                Some(Expression::Literal(Literal::String(StringLiteral {
                    value: lit.value.to_string(),
                })))
            }
            oxc::Expression::BooleanLiteral(lit) => {
                Some(Expression::Literal(Literal::Boolean(BooleanLiteral {
                    value: lit.value,
                })))
            }
            oxc::Expression::NullLiteral(_) => {
                Some(Expression::Literal(Literal::Null))
            }
            oxc::Expression::BinaryExpression(expr) => {
                let left = Box::new(Expression::from_oxc(&expr.left)?);
                let right = Box::new(Expression::from_oxc(&expr.right)?);
                let operator = BinaryOperator::from_oxc(expr.operator)?;

                Some(Expression::BinaryExpression {
                    left,
                    operator,
                    right,
                })
            }
            oxc::Expression::TemplateLiteral(tmpl) => {
                let quasis = tmpl.quasis.iter()
                    .map(|quasi| TemplateElement {
                        value: quasi.value.raw.to_string(),
                        tail: quasi.tail,
                    })
                    .collect();
                
                let expressions = tmpl.expressions.iter()
                    .filter_map(|expr| Expression::from_oxc(expr))
                    .collect();
                
                Some(Expression::TemplateLiteral { quasis, expressions })
            }
            oxc::Expression::FunctionExpression(func) => {
                Some(Expression::FunctionExpression(FunctionExpression::from_oxc(func)?))
            }
            oxc::Expression::CallExpression(call) => {
                let callee = Box::new(Expression::from_oxc(&call.callee)?);
                let arguments = call.arguments.iter()
                    .filter_map(|arg| {
                        if let Some(expr) = arg.as_expression() {
                            Expression::from_oxc(expr)
                        } else {
                            None // Skip spread arguments for now
                        }
                    })
                    .collect();
                
                Some(Expression::CallExpression { callee, arguments })
            }
            oxc::Expression::RegExpLiteral(regex) => {
                Some(Expression::Literal(Literal::RegExp(RegExpLiteral {
                    pattern: regex.regex.pattern.to_string(),
                    flags: regex.regex.flags.to_string(),
                })))
            }
            // TODO: Add more expression types as needed
            _ => None,
        }
    }
}

impl Pattern {
    /// Convert from OXC BindingPattern to our Pattern type
    pub fn from_oxc(oxc_pattern: &oxc::BindingPattern<'_>) -> Option<Self> {
        match &oxc_pattern.kind {
            oxc::BindingPatternKind::BindingIdentifier(id) => {
                Some(Pattern::Identifier(Identifier::from_oxc(id)))
            }
            // TODO: Add more pattern types as needed
            _ => None,
        }
    }
}

impl Identifier {
    /// Convert from OXC BindingIdentifier to our Identifier type
    pub fn from_oxc(oxc_id: &oxc::BindingIdentifier<'_>) -> Self {
        Self {
            name: oxc_id.name.to_string(),
        }
    }
}

impl BinaryOperator {
    /// Convert from OXC BinaryOperator to our BinaryOperator type
    pub fn from_oxc(oxc_op: oxc::BinaryOperator) -> Option<Self> {
        match oxc_op {
            oxc::BinaryOperator::Addition => Some(BinaryOperator::Add),
            oxc::BinaryOperator::Subtraction => Some(BinaryOperator::Subtract),
            oxc::BinaryOperator::Multiplication => Some(BinaryOperator::Multiply),
            oxc::BinaryOperator::Division => Some(BinaryOperator::Divide),
            oxc::BinaryOperator::Remainder => Some(BinaryOperator::Remainder),
            oxc::BinaryOperator::Exponential => Some(BinaryOperator::Exponentiation),
            oxc::BinaryOperator::Equality => Some(BinaryOperator::Equal),
            oxc::BinaryOperator::Inequality => Some(BinaryOperator::NotEqual),
            oxc::BinaryOperator::StrictEquality => Some(BinaryOperator::StrictEqual),
            oxc::BinaryOperator::StrictInequality => Some(BinaryOperator::StrictNotEqual),
            oxc::BinaryOperator::LessThan => Some(BinaryOperator::LessThan),
            oxc::BinaryOperator::LessEqualThan => Some(BinaryOperator::LessThanEqual),
            oxc::BinaryOperator::GreaterThan => Some(BinaryOperator::GreaterThan),
            oxc::BinaryOperator::GreaterEqualThan => Some(BinaryOperator::GreaterThanEqual),
            oxc::BinaryOperator::ShiftLeft => Some(BinaryOperator::LeftShift),
            oxc::BinaryOperator::ShiftRight => Some(BinaryOperator::RightShift),
            oxc::BinaryOperator::ShiftRightZeroFill => Some(BinaryOperator::UnsignedRightShift),
            oxc::BinaryOperator::BitwiseAnd => Some(BinaryOperator::BitwiseAnd),
            oxc::BinaryOperator::BitwiseOR => Some(BinaryOperator::BitwiseOr),
            oxc::BinaryOperator::BitwiseXOR => Some(BinaryOperator::BitwiseXor),
            oxc::BinaryOperator::In => Some(BinaryOperator::In),
            oxc::BinaryOperator::Instanceof => Some(BinaryOperator::Instanceof),
        }
    }
}

impl PropertyKey {
    /// Convert from OXC PropertyKey to our PropertyKey type
    pub fn from_oxc(oxc_key: &oxc::PropertyKey<'_>) -> Option<Self> {
        match oxc_key {
            oxc::PropertyKey::Identifier(id) => {
                Some(PropertyKey::Identifier(Identifier {
                    name: id.name.to_string(),
                }))
            }
            oxc::PropertyKey::PrivateIdentifier(private) => {
                Some(PropertyKey::PrivateName(PrivateName {
                    name: private.name.to_string(),
                }))
            }
            _ => {
                // For now, handle computed property keys as identifier
                Some(PropertyKey::Identifier(Identifier {
                    name: "computed".to_string(),
                }))
            }
        }
    }
}

impl FunctionExpression {
    /// Convert from OXC Function to our FunctionExpression type
    pub fn from_oxc(oxc_func: &oxc::Function<'_>) -> Option<Self> {
        let id = oxc_func.id.as_ref().map(|id| Identifier {
            name: id.name.to_string(),
        });
        
        let params = oxc_func.params.items.iter()
            .filter_map(|param| Pattern::from_oxc(&param.pattern))
            .collect();
        
        let body = BlockStatement {
            body: oxc_func.body.as_ref()?
                .statements.iter()
                .filter_map(|stmt| Statement::from_oxc(stmt))
                .collect(),
        };
        
        Some(FunctionExpression {
            id,
            params,
            body,
            is_async: oxc_func.r#async,
            is_generator: oxc_func.generator,
        })
    }
}