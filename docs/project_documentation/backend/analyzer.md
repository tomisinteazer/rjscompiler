# Scope Analyzer Component

## Overview

The analyzer component performs semantic analysis of the JavaScript AST, including scope construction, symbol binding, capture detection, and safety classification for minification. It enriches the AST with scope and semantic metadata to ensure all variables, functions, and classes are correctly resolved, tracks closures, exports, and references, and records constructs that impact optimization safety.

## Phase Details

- **Phase**: Phase 3 — Analysis
- **Approach**: TDD (Test Driven Development)
- **Objective**: Perform semantic analysis of the JavaScript AST, including scope construction, symbol binding, capture detection, and safety classification for minification
- **Description**: The analysis phase enriches the AST with scope and semantic metadata. It ensures all variables, functions, and classes are correctly resolved, tracks closures, exports, and references, and records constructs (eval, with, this) that impact optimization safety.

## Key Responsibilities

- **Scope Analysis**: Build hierarchical scope tree for variable resolution
- **Symbol Table**: Track all identifiers and their bindings with detailed metadata
- **Reference Tracking**: Map variable uses to declarations with read/write classification
- **Capture Detection**: Identify closure captures for safe minification
- **Safety Classification**: Flag scopes and symbols that cannot be safely renamed
- **Conflict Detection**: Identify naming collisions before renaming

## TDD Development Approach

### Step 1: Define Test Suites

Before implementing, create comprehensive test cases for all analyzer functionality.

#### Scope Builder Tests

| Test Name | Input Code | Expected Scope Structure |
|-----------|------------|-------------------------|
| Simple Function Scope | `function foo() { let x = 1; }` | Global scope → Function scope(foo) with binding x |
| Nested Functions | `function outer() { function inner() {} }` | Global → Function(outer) → Function(inner) |
| Block Scoping | `if (true) { let blockVar = 1; }` | Global scope with block child scope containing blockVar |
| Variable Shadowing | `let x = 1; function foo() { let x = 2; }` | Global x, Function foo with shadowed x |
| Closure Capture | `function outer() { let x = 1; function inner() { return x; } }` | outer.x marked as captured |

#### Semantic Analysis Tests

| Test Name | Input Code | Expected Semantic Flags |
|-----------|------------|------------------------|
| Eval Detection | `function evil() { eval('x'); }` | Function scope marked unsafe due to eval |
| With Statement | `function bad() { with(obj) { prop; } }` | Function scope marked unsafe due to with |
| Arrow This | `const f = () => this.prop;` | This usage marked as lexical |
| Function This | `function f() { return this.prop; }` | This usage marked as dynamic |

#### Edge Case Tests

| Test Name | Input Code | Expected Behavior |
|-----------|------------|------------------|
| Hoisting | `console.log(x); var x = 5;` | Var x declared in function scope before usage |
| TDZ Violation | `console.log(x); let x = 5;` | Temporal dead zone reference detected |
| Module Exports | `export const value = 42;` | Value marked as exported in module scope |

### Step 2: Implement Minimal Analyzer Wrapper

Write a wrapper function that takes the AST as input and returns analysis results.

#### Requirements:
- Function signature: `analyze_ast(ast: &AST) -> Result<AnalysisResult, AnalysisError>`
- AnalysisResult should contain symbol table, semantic info, and annotations
- Results should be serializable for debugging and testing
- Error handling with descriptive messages

### Step 3: Run Tests Incrementally

For each test case, run analyzer against input and compare against expected results.

#### Validation Checks:
- Scope hierarchy matches expected structure
- Symbol bindings are correctly recorded
- References are properly resolved
- Captures are accurately detected
- Safety flags are correctly set
- Edge cases are handled appropriately

### Step 4: Refine Analysis Implementation

Enhance the analyzer with additional features and optimizations based on test feedback.

#### Requirements:
- Handle all JavaScript language features
- Optimize performance for large codebases
- Provide detailed error reporting
- Maintain compatibility with downstream components

### Step 5: Finalize & Document

Ensure analysis phase outputs are well-defined and comprehensively tested.

#### Final Deliverables:
- Complete symbol table with all binding information
- Semantic flags for optimization safety
- Annotated AST with scope metadata
- Comprehensive test suite covering all scenarios
- Developer documentation explaining the analysis process

## Inputs and Outputs

### Inputs
- **AST**: The fully parsed JavaScript Abstract Syntax Tree from Phase 2 (Parsing)
- **Parser Metadata**: Token positions, comments, and contextual hints passed from parsing

### Outputs
- **Symbol Table**: Hierarchical mapping of all scopes, bindings, and references
- **Semantic Info**: Flags for eval, with, this usage and scope safety classification
- **Analysis Annotations**: Decorated AST nodes with scope/symbol metadata
- **Report**: Structured JSON report for debugging, TDD validation, and downstream phases

## Core Components

### Scope Builder (Component ID: 6)

#### Responsibilities
- Construct a scope tree with unique scope IDs and parent-child relationships
- Enter new scope at functions, blocks, catch clauses, classes, modules, etc.
- Maintain symbol bindings for variables, functions, classes, parameters
- Track references to identifiers (read/write/declare)
- Resolve shadowing and redeclarations correctly
- Detect captures: mark when an inner scope closes over an outer variable
- Track exports in module/global scope

#### Scope Data Model
```json
{
  "Scope": {
    "scope_id": "string|integer (unique per scope)",
    "type": "global|function|block|class|module|catch|with",
    "parent_scope_id": "string|integer|null",
    "bindings": [
      {
        "identifier": "string",
        "kind": "variable|function|class|parameter|import|export",
        "declared_at": "AST node location",
        "references": ["AST node locations"],
        "captured": "boolean",
        "exported": "boolean"
      }
    ],
    "children": ["scope_id references"]
  }
}
```

#### Edge Cases Handled
- Shadowed variables inside nested blocks
- Function parameters shadowing outer variables
- Global scope references with and without 'var'
- Block-scoped let/const in loops
- Class names inside their own body

### Semantic Info (Component ID: 7)

#### Responsibilities
- Detect eval usage → mark containing and ancestor scopes unsafe for renaming
- Detect with usage → mark scope resolution as dynamic and unsafe
- Detect this usage → classify as lexical (arrow function) or dynamic (regular function)
- Propagate unsafe flags upward if eval/with/this affect parent scopes
- Mark scope safety classification: safe vs unsafe

#### Semantic Flags Data Model
```json
{
  "SemanticFlags": {
    "scope_id": "string|integer",
    "has_eval": "boolean",
    "has_with": "boolean",
    "uses_this": "boolean",
    "this_type": "dynamic|lexical|global",
    "safety": "safe|unsafe"
  }
}
```

#### Edge Cases Handled
- Eval inside strict mode vs sloppy mode
- Arrow function this binding vs function expression this
- With statement inside nested scopes
- Indirect eval (e.g., (0, eval)()) vs direct eval
- Modules with top-level this (should be undefined in strict mode)

## Scope Analysis

### Scope Types
- **Global Scope**: Top-level variables and functions
- **Function Scope**: Parameters and local variables
- **Block Scope**: Let/const declarations in blocks
- **Module Scope**: Import/export bindings
- **Class Scope**: Class declarations and expressions
- **Catch Scope**: Exception parameter scope
- **With Scope**: Dynamic scope from with statements

### Hoisting Behavior
- **Variable Hoisting**: Var declarations hoisted to function scope
- **Function Hoisting**: Function declarations hoisted completely
- **Temporal Dead Zone**: Let/const variables before declaration
- **Parameter Initialization**: Default parameter scope interactions

## Symbol Table Structure

### Symbol Information
- **Name**: Original identifier name
- **Type**: Variable, function, parameter, property, class, import, export
- **Scope**: Which scope the symbol belongs to
- **Bindings**: All locations where symbol is bound
- **References**: All locations where symbol is used
- **Captured**: Boolean flag for closure capture
- **Exported**: Boolean flag for module exports
- **Safety**: Safe/unsafe classification for renaming

### Binding Types
- **Declaration**: Variable, function, class declarations
- **Parameter**: Function and catch clause parameters
- **Import**: Module import bindings
- **Property**: Object property names (when safe to rename)
- **Export**: Module export bindings

## Reference Analysis

### Reference Tracking
- **Read References**: Variable usage in expressions
- **Write References**: Assignment and modification
- **Call References**: Function invocations
- **Property Access**: Member expression tracking

### Dynamic Access Detection
- **Bracket Notation**: `obj['property']` - unsafe for renaming
- **Computed Properties**: `obj[variable]` - dynamic access
- **Eval Usage**: Dynamic code execution - disable renaming
- **With Statements**: Avoid renaming in with blocks
- **Indirect References**: `window['variable']` - potential global access

## Renaming Safety Analysis

### Safe Renaming Conditions
- **No external references**: Symbol not accessed from outside its scope
- **No dynamic access**: Not accessed via bracket notation or eval
- **No eval interaction**: Not referenced in eval'd code
- **Proper scoping**: Rename doesn't create conflicts
- **Safe scope**: Containing scope marked as safe for renaming

### Conflict Prevention
- **Reserved words**: Avoid JavaScript keywords
- **Built-in objects**: Don't shadow global objects
- **Scope conflicts**: Ensure renamed symbols don't collide
- **Cross-scope references**: Maintain reference integrity
- **Export preservation**: Keep exported symbols unchanged when necessary

## Analysis Pipeline

1. Traverse AST depth-first
2. At scope entry: create scope object and link to parent
3. At declaration: bind identifier in current scope
4. At identifier reference: resolve scope chain, record reference
5. At closure: mark captured variables
6. At export: tag symbol as exported
7. At eval/with/this: annotate semantic flags
8. Finalize: classify scopes as safe or unsafe for renaming

## Algorithm Implementation

### Scope Tree Building
```rust
// Pseudo-code for scope analysis
fn analyze_scope(node: &AstNode, scope: &mut Scope) {
    match node {
        AstNode::FunctionDeclaration(func) => {
            let child_scope = Scope::new(scope);
            for param in &func.parameters {
                child_scope.declare(param.name, SymbolType::Parameter);
            }
            analyze_scope(&func.body, &mut child_scope);
        }
        AstNode::VariableDeclaration(var) => {
            scope.declare(var.name, SymbolType::Variable);
        }
        // ... handle other node types
    }
}
```

### Symbol Renaming Strategy
- **Frequency-based**: Rename most-used symbols to shortest names
- **Scope-aware**: Use same short name in different scopes
- **Collision avoidance**: Check conflicts before assignment
- **Preservation**: Keep symbols that must remain unchanged
- **Safety-first**: Only rename symbols in safe scopes

## Integration Points

- **Input**: Abstract Syntax Tree from parser (Phase 2)
- **Output**: Annotated AST with scope and symbol information
- **Dependencies**: Parser component
- **Used By**: Transformer component for safe renaming

## Performance Optimizations

- **Single-pass analysis**: Minimize AST traversals
- **Incremental updates**: Reanalyze only changed portions
- **Memory efficiency**: Compact symbol table representation
- **Fast lookups**: Hash-based symbol resolution
- **Lazy evaluation**: Compute expensive analyses only when needed

## Edge Cases

### JavaScript Peculiarities
- **Variable hoisting**: Var declarations move to function top
- **Function hoisting**: Function declarations fully hoisted
- **This binding**: Context-dependent this references
- **Closures**: Variables captured by inner functions
- **Exception handling**: Catch clause scope behavior

### Modern JavaScript Features
- **Arrow functions**: Lexical this binding
- **Destructuring**: Complex binding patterns
- **Default parameters**: Parameter scope interactions
- **Rest/spread**: Variable argument handling
- **Modules**: Import/export binding semantics
- **Classes**: Static and instance member scoping
- **Private fields**: `#field` syntax scoping rules

## TDD Strategy

### Unit Tests
- Verify scope creation for nested functions, blocks, classes, and modules
- Check shadowing: ensure inner scope hides outer scope variable
- Validate closure detection: inner function referencing outer variable
- Check correct export marking in ES modules
- Test eval disables renaming within its scope and ancestors
- Ensure with marks scope as unsafe
- Verify this is tracked differently in arrow functions vs regular functions

### Integration Tests
- Analyze nested closures with multiple captures and validate captured flags
- Run analysis on modules with multiple exports and imports
- Check eval inside strict vs non-strict modes
- Analyze real-world snippets with with/eval/this to confirm correct scope safety classification
- Cross-validate that unsafe scopes prevent minifier from renaming identifiers

### Test-Driven Development Workflow
1. **Write failing tests** for new functionality
2. **Implement minimal code** to pass tests
3. **Refactor** while keeping tests passing
4. **Add more tests** for edge cases
5. **Repeat** until feature is complete

## Success Criteria

- Every identifier maps to exactly one scope and definition
- Captured variables correctly marked in closures
- Unsafe scopes flagged correctly for eval/with/this
- Exports and imports tracked for module scope
- Analysis annotations align with downstream minifier expectations

---

*Status*: 🚧 In Development  
*Owner*: JavaScript Minifier Team  
*Last Updated*: 2025-08-25