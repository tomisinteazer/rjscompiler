# Rust Performance Optimization Standards

## General Optimization Principles

### Profile Before Optimizing
- **Measure first**: Use profiling tools to identify actual bottlenecks
- **Benchmark**: Establish baseline performance metrics
- **Focus on hot paths**: Optimize the code that runs most frequently
- **Avoid premature optimization**: Don't optimize without evidence of need

### Performance Guidelines

#### Efficient Collections
Choose the right data structure for your use case:

```rust
use std::collections::{HashMap, BTreeMap, HashSet, VecDeque};
use indexmap::IndexMap; // When insertion order matters
use ahash::AHashMap;    // Faster hashing for non-cryptographic use

// Good: Use appropriate collection types
pub struct SymbolTable {
    // HashMap for O(1) lookups when order doesn't matter
    symbols: AHashMap<String, Symbol>,
    
    // Vec for ordered access and cache-friendly iteration
    symbol_order: Vec<String>,
    
    // BTreeMap when you need sorted iteration
    sorted_scopes: BTreeMap<usize, Scope>,
}

// Good: Pre-allocate when size is known
pub fn process_tokens(tokens: &[Token]) -> Vec<ProcessedToken> {
    let mut result = Vec::with_capacity(tokens.len());
    
    for token in tokens {
        result.push(process_token(token));
    }
    
    result
}
```

#### Iterator Usage
Prefer iterators over collecting when possible:

```rust
// Good: Use iterators for lazy evaluation
pub fn find_unused_variables(ast: &AstNode) -> impl Iterator<Item = &Variable> + '_ {
    ast.walk_nodes()
        .filter_map(|node| node.as_variable())
        .filter(|var| !var.is_used())
}

// Good: Chain operations without intermediate collections
pub fn minify_identifiers(identifiers: &[String]) -> Vec<String> {
    identifiers
        .iter()
        .enumerate()
        .map(|(index, _)| generate_short_name(index))
        .collect()
}

// Avoid: Unnecessary intermediate collections
pub fn process_identifiers_bad(identifiers: &[String]) -> Vec<String> {
    let filtered: Vec<_> = identifiers
        .iter()
        .filter(|id| !id.is_empty())
        .collect();  // Unnecessary collection
    
    filtered
        .iter()
        .map(|id| id.to_lowercase())
        .collect()
}
```

#### Memory Management
Minimize allocations and optimize memory usage:

```rust
use std::borrow::Cow;

// Good: Use Cow for conditional cloning
pub fn normalize_identifier<'a>(name: &'a str) -> Cow<'a, str> {
    if name.chars().all(|c| c.is_ascii_lowercase()) {
        Cow::Borrowed(name)  // No allocation needed
    } else {
        Cow::Owned(name.to_lowercase())  // Allocate only when necessary
    }
}

// Good: Use string interning for frequently used strings
use string_cache::DefaultAtom;

pub struct OptimizedParser {
    // Intern common keywords to avoid repeated allocations
    keywords: HashSet<DefaultAtom>,
}

impl OptimizedParser {
    pub fn is_keyword(&self, name: &str) -> bool {
        let atom = DefaultAtom::from(name);
        self.keywords.contains(&atom)
    }
}

// Good: Use SmallVec for stack allocation of small collections
use smallvec::{SmallVec, smallvec};

// Most function parameters lists are small, use stack allocation
type ParameterList = SmallVec<[Parameter; 4]>;

pub fn analyze_function_parameters(params: &ParameterList) -> AnalysisResult {
    // Stack-allocated for small parameter lists
    let mut analysis = smallvec![];
    
    for param in params {
        analysis.push(analyze_parameter(param));
    }
    
    AnalysisResult::new(analysis)
}
```

## Async and Concurrency Optimization

### Parallel Processing
Use Rayon for data parallelism:

```rust
use rayon::prelude::*;

// Good: Parallel processing for independent operations
pub fn minify_files_parallel(files: &[PathBuf]) -> Vec<Result<String, MinifyError>> {
    files
        .par_iter()  // Parallel iterator
        .map(|path| minify_file(path))
        .collect()
}

// Good: Parallel transformation of AST nodes
pub fn transform_ast_parallel(nodes: &mut [AstNode]) {
    nodes
        .par_iter_mut()
        .for_each(|node| transform_node(node));
}

// Good: Use parallel reduce for aggregation
pub fn count_variables_parallel(files: &[AstNode]) -> usize {
    files
        .par_iter()
        .map(|ast| count_variables_in_ast(ast))
        .sum()
}
```

### Async Operations
Efficient async patterns for I/O:

```rust
use tokio::fs;
use futures::stream::{self, StreamExt};

// Good: Concurrent file processing
pub async fn process_files_async(paths: Vec<PathBuf>) -> Vec<Result<String, ProcessError>> {
    let futures = paths.into_iter().map(|path| async move {
        let content = fs::read_to_string(&path).await?;
        minify_content(&content)
    });
    
    // Process up to 10 files concurrently
    stream::iter(futures)
        .buffered(10)
        .collect()
        .await
}

// Good: Streaming large file processing
pub async fn process_large_file_stream(
    reader: impl AsyncRead + Unpin,
) -> Result<impl Stream<Item = ProcessedChunk>, ProcessError> {
    let chunks = tokio_util::codec::FramedRead::new(reader, ChunkCodec::new());
    
    Ok(chunks.map(|chunk| process_chunk(chunk)))
}
```

## Cache-Friendly Patterns

### Data Layout Optimization
```rust
// Good: Structure of Arrays (SoA) for better cache locality
pub struct OptimizedSymbolTable {
    names: Vec<String>,
    types: Vec<SymbolType>,
    scopes: Vec<ScopeId>,
    is_used: Vec<bool>,
}

impl OptimizedSymbolTable {
    // Process all symbols of a specific type efficiently
    pub fn process_variables(&mut self) {
        for i in 0..self.types.len() {
            if self.types[i] == SymbolType::Variable {
                // Cache-friendly: access consecutive memory
                self.process_variable_at_index(i);
            }
        }
    }
}

// Avoid: Array of Structures (AoS) can be less cache-friendly for bulk operations
pub struct Symbol {
    name: String,
    symbol_type: SymbolType,
    scope: ScopeId,
    is_used: bool,
}
```

### Memory Pool Allocation
```rust
use bumpalo::Bump;

pub struct AstArena {
    arena: Bump,
}

impl AstArena {
    pub fn new() -> Self {
        Self {
            arena: Bump::new(),
        }
    }
    
    // Good: Allocate AST nodes in arena for better cache locality
    pub fn alloc_node(&self, kind: NodeKind) -> &mut AstNode {
        self.arena.alloc(AstNode::new(kind))
    }
    
    // All nodes freed together when arena is dropped
}

// Usage: Process entire AST with good cache locality
pub fn optimize_ast_with_arena(source: &str) -> Result<String, ParseError> {
    let arena = AstArena::new();
    let ast = parse_with_arena(source, &arena)?;
    let optimized = optimize_ast(ast);
    generate_code(optimized)
    // Arena automatically freed here
}
```

## Algorithm Optimization

### Efficient String Operations
```rust
// Good: Use string builders for incremental construction
use std::fmt::Write;

pub fn build_minified_output(nodes: &[AstNode]) -> String {
    let estimated_size = nodes.len() * 50; // Rough estimate
    let mut output = String::with_capacity(estimated_size);
    
    for node in nodes {
        write!(output, "{}", node.minified_representation()).unwrap();
    }
    
    output
}

// Good: Use rope data structure for large text manipulations
use ropey::Rope;

pub fn apply_large_scale_transformations(source: &str, transforms: &[Transform]) -> String {
    let mut rope = Rope::from_str(source);
    
    // Apply transformations in reverse order to maintain indices
    for transform in transforms.iter().rev() {
        rope.remove(transform.start..transform.end);
        rope.insert(transform.start, &transform.replacement);
    }
    
    rope.to_string()
}
```

### Optimized Parsing Algorithms
```rust
// Good: Use zero-copy parsing when possible
use std::borrow::Cow;

pub struct ZeroCopyToken<'a> {
    kind: TokenKind,
    text: &'a str,  // Reference to original source
    span: Span,
}

impl<'a> ZeroCopyToken<'a> {
    pub fn as_owned(&self) -> OwnedToken {
        OwnedToken {
            kind: self.kind,
            text: self.text.to_owned(),
            span: self.span,
        }
    }
}

// Good: Incremental parsing for large files
pub struct IncrementalParser {
    cache: HashMap<FileId, ParseResult>,
    dependency_graph: DependencyGraph,
}

impl IncrementalParser {
    pub fn parse_with_cache(&mut self, file_id: FileId, source: &str) -> ParseResult {
        if let Some(cached) = self.cache.get(&file_id) {
            if !self.has_dependencies_changed(file_id) {
                return cached.clone();
            }
        }
        
        let result = self.parse_full(source);
        self.cache.insert(file_id, result.clone());
        result
    }
}
```

## Micro-optimizations

### Bit Manipulation
```rust
// Good: Use bit flags for boolean combinations
#[derive(Clone, Copy)]
pub struct OptimizationFlags(u32);

impl OptimizationFlags {
    pub const RENAME_VARIABLES: Self = Self(1 << 0);
    pub const REMOVE_DEAD_CODE: Self = Self(1 << 1);
    pub const INLINE_FUNCTIONS: Self = Self(1 << 2);
    pub const CONSTANT_FOLDING: Self = Self(1 << 3);
    
    pub fn contains(self, flag: Self) -> bool {
        (self.0 & flag.0) != 0
    }
    
    pub fn insert(&mut self, flag: Self) {
        self.0 |= flag.0;
    }
}

// Usage: Efficient flag checking
pub fn apply_optimizations(ast: &mut AstNode, flags: OptimizationFlags) {
    if flags.contains(OptimizationFlags::RENAME_VARIABLES) {
        rename_variables(ast);
    }
    
    if flags.contains(OptimizationFlags::REMOVE_DEAD_CODE) {
        remove_dead_code(ast);
    }
}
```

### SIMD Operations (when applicable)
```rust
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

// Good: Use SIMD for parallel character processing
#[cfg(target_arch = "x86_64")]
pub fn fast_char_classification(input: &[u8]) -> Vec<CharClass> {
    let mut result = Vec::with_capacity(input.len());
    
    // Process 16 bytes at a time with SIMD
    for chunk in input.chunks_exact(16) {
        unsafe {
            let chars = _mm_loadu_si128(chunk.as_ptr() as *const __m128i);
            let is_alpha = _mm_and_si128(
                _mm_cmpgt_epi8(chars, _mm_set1_epi8(b'@' as i8)),
                _mm_cmplt_epi8(chars, _mm_set1_epi8(b'[' as i8)),
            );
            
            // Process SIMD results...
        }
    }
    
    // Handle remaining bytes with scalar code
    for &byte in input.chunks_exact(16).remainder() {
        result.push(classify_char_scalar(byte));
    }
    
    result
}
```

## Benchmarking and Profiling Integration

### Criterion Benchmarks
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

fn benchmark_minification_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("minification");
    
    for size in [1_000, 10_000, 100_000, 1_000_000].iter() {
        let source = generate_test_javascript(*size);
        
        group.bench_with_input(
            BenchmarkId::new("minify", size),
            &source,
            |b, source| {
                b.iter(|| minify_javascript(black_box(source)))
            },
        );
    }
    
    group.finish();
}

criterion_group!(benches, benchmark_minification_sizes);
criterion_main!(benches);
```

### Performance Regression Detection
```rust
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn minification_performance_regression() {
        let large_file = include_str!("../test_data/large_file.js");
        
        let start = Instant::now();
        let _result = minify_javascript(large_file).unwrap();
        let duration = start.elapsed();
        
        // Fail if performance regresses beyond acceptable threshold
        assert!(
            duration < Duration::from_millis(100),
            "Minification took too long: {:?} (max: 100ms)",
            duration
        );
    }
}
```