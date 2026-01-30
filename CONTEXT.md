# Mendes Programming Language - Project Context

## Overview

**Mendes** is a compiled programming language designed for high-performance backend development. It features native HTTP support, real async/await, and a Rust-inspired ownership system.

- **File Extension**: `.ms`
- **Target**: Native binaries via Rust code generation
- **Paradigm**: Imperative with functional elements
- **Key Feature**: Significant indentation (Python-like syntax)

---

## Architecture

### Compilation Pipeline

```
Source Code (.ms)
       │
       ▼
   ┌───────┐
   │ Lexer │  → Tokens (with INDENT/DEDENT)
   └───┬───┘
       │
       ▼
   ┌────────┐
   │ Parser │  → Abstract Syntax Tree (AST)
   └───┬────┘
       │
       ▼
   ┌──────────┐
   │ Semantic │  → Type-checked AST + Symbol Tables
   └────┬─────┘
       │
       ▼
   ┌────┐
   │ IR │  → Intermediate Representation
   └─┬──┘
       │
       ▼
   ┌─────────┐
   │ Codegen │  → Rust source code
   └────┬────┘
       │
       ▼
   ┌───────┐
   │ Cargo │  → Native binary (.exe)
   └───────┘
```

### Crate Structure

```
Linguagem-Mendes/
├── Cargo.toml                    # Workspace root
├── crates/
│   ├── mendes-lexer/            # Tokenization
│   │   ├── src/lib.rs           # Module exports
│   │   ├── src/token.rs         # Token definitions
│   │   └── src/lexer.rs         # Lexer implementation
│   │
│   ├── mendes-parser/           # Parsing
│   │   ├── src/lib.rs           # Module exports
│   │   ├── src/ast.rs           # AST node definitions
│   │   └── src/parser.rs        # Recursive descent parser
│   │
│   ├── mendes-semantic/         # Semantic analysis
│   │   ├── src/lib.rs           # Module exports
│   │   ├── src/analyzer.rs      # Main analyzer
│   │   ├── src/types.rs         # Type system
│   │   ├── src/symbols.rs       # Symbol table
│   │   └── src/ownership.rs     # Ownership checker
│   │
│   ├── mendes-ir/               # Intermediate representation
│   │   ├── src/lib.rs           # Module exports
│   │   ├── src/ir.rs            # IR node definitions
│   │   └── src/lower.rs         # AST → IR lowering
│   │
│   ├── mendes-codegen/          # Code generation
│   │   ├── src/lib.rs           # Module exports
│   │   ├── src/rust_gen.rs      # Rust backend (primary)
│   │   └── src/c_gen.rs         # C backend (secondary)
│   │
│   ├── mendes-error/            # Error handling
│   │   ├── src/lib.rs           # Module exports
│   │   ├── src/diagnostic.rs    # Diagnostic types
│   │   └── src/span.rs          # Source location tracking
│   │
│   ├── mendes-runtime/          # Runtime library
│   │   └── src/lib.rs           # HTTP, DB, WebSocket runtime
│   │
│   └── mendes-cli/              # Command-line interface
│       └── src/main.rs          # CLI entry point
│
├── examples/                     # Example programs
│   ├── minimal.ms               # Minimal example
│   ├── hello.ms                 # Hello world
│   ├── api_basic.ms             # Basic HTTP API
│   └── ...
│
├── tests/                        # Integration tests
│   └── integration_tests.rs     # 48 test cases
│
└── docs/                         # Documentation
    ├── tutorial.md              # Step-by-step tutorial
    ├── language-reference.md    # Language specification
    ├── cli-reference.md         # CLI documentation
    ├── runtime-api.md           # Runtime API docs
    ├── architecture.md          # Compiler internals
    ├── examples.md              # Annotated examples
    └── grammar.md               # Formal EBNF grammar
```

---

## Language Features

### 1. Significant Indentation

Uses indentation instead of braces (like Python):

```mendes
fn factorial(n: int) -> int:
    if n <= 1:
        return 1
    return n * factorial(n - 1)
```

The lexer tracks indentation levels and emits `INDENT`/`DEDENT` tokens.

### 2. Type System

**Primitive Types:**
- `int` - 64-bit signed integer
- `float` - 64-bit floating point
- `bool` - Boolean (true/false)
- `string` - UTF-8 string
- `()` - Unit type

**Composite Types:**
- `[T]` - Array of T
- `(T, U, ...)` - Tuple
- `Option<T>` - Optional value (Some/None)
- `Result<T, E>` - Result type (Ok/Err)
- `Map<K, V>` - Hash map

**User-Defined:**
- `struct` - Data structures
- `enum` - Sum types with variants
- `trait` - Interfaces

### 3. Ownership System

Rust-inspired ownership with three states:
- **Owned** - Value has single owner
- **Borrowed** (`&T`) - Immutable reference
- **Mutable Borrow** (`&mut T`) - Mutable reference

```mendes
let user = User { name: "Alice" }
process(&user)      // Borrow
modify(&mut user)   // Mutable borrow
consume(user)       // Move ownership
```

### 4. Native HTTP

First-class HTTP support:

```mendes
server:
    host "127.0.0.1"
    port 8080

api GET /users:
    let users = db.query("SELECT * FROM users")
    return json(users)

api POST /users:
    let user = req.json::<User>()
    db.insert("users", &user)
    return json(user).status(201)
```

### 5. Async/Await

Real async operations (not threads):

```mendes
async fn fetch_data() -> Result<Data, Error>:
    let response = await http.get("https://api.example.com/data")
    return response.json()
```

### 6. Database Integration

Built-in connection pooling:

```mendes
db:
    driver "postgres"
    host "localhost"
    database "myapp"
    pool_size 10

api GET /products:
    let products = await db.query("SELECT * FROM products")
    return json(products)
```

### 7. Pattern Matching

Exhaustive pattern matching:

```mendes
match result:
    Ok(value):
        print("Success: {value}")
    Err(e):
        print("Error: {e}")
```

### 8. Generics

Parametric polymorphism:

```mendes
fn first<T>(items: [T]) -> Option<T>:
    if items.len() > 0:
        return Some(items[0])
    return None
```

---

## Token Types

```rust
pub enum TokenKind {
    // Keywords
    Let, Mut, Fn, Return, If, Else, For, In, While,
    Struct, Enum, Impl, Trait, Api, Server, Middleware,
    Db, Async, Await, Match, Use, Pub, Mod, Import,
    True, False, None, Some, Ok, Err,

    // HTTP Methods
    Get, Post, Put, Delete, Patch,

    // Types
    IntType, FloatType, BoolType, StringType,

    // Literals
    IntLit(i64),
    FloatLit(f64),
    StringLit(String),

    // Identifiers
    Ident(String),

    // Operators
    Plus, Minus, Star, Slash, Percent,
    Eq, EqEq, Ne, Lt, Le, Gt, Ge,
    And, Or, Not, Ampersand, AmpersandMut,
    Arrow, FatArrow, DoubleColon, Dot, DotDot,

    // Delimiters
    Colon, Comma, Semicolon,
    LParen, RParen, LBrace, RBrace, LBracket, RBracket,
    LAngle, RAngle,

    // Indentation
    Newline, Indent, Dedent,

    // Special
    Eof, Error(String),
}
```

---

## AST Structure

### Statements

```rust
pub enum Stmt {
    Let { name: String, ty: Option<Type>, value: Option<Expr>, mutable: bool },
    Fn { name: String, generics: Vec<String>, params: Vec<Param>,
         ret_type: Option<Type>, is_async: bool, body: Vec<Stmt>, is_pub: bool },
    Struct { name: String, generics: Vec<String>, fields: Vec<Field>, is_pub: bool },
    Enum { name: String, generics: Vec<String>, variants: Vec<Variant>, is_pub: bool },
    Impl { target: String, generics: Vec<String>, trait_name: Option<String>,
           methods: Vec<Stmt> },
    Trait { name: String, generics: Vec<String>, methods: Vec<TraitMethod>, is_pub: bool },
    Api { method: HttpMethod, path: String, params: Vec<PathParam>,
          is_async: bool, body: Vec<Stmt> },
    Server { config: ServerConfig },
    Db { config: DbConfig },
    Middleware { name: String, body: Vec<Stmt> },
    If { condition: Expr, then_body: Vec<Stmt>, else_body: Option<Vec<Stmt>> },
    For { var: String, iter: Expr, body: Vec<Stmt> },
    While { condition: Expr, body: Vec<Stmt> },
    Match { expr: Expr, arms: Vec<MatchArm> },
    Return(Option<Expr>),
    Break,
    Continue,
    Use { path: Vec<String>, alias: Option<String> },
    Mod { name: String, is_pub: bool },
    Expr(Expr),
}
```

### Expressions

```rust
pub enum Expr {
    Literal(Literal),
    Ident(String),
    Binary { left: Box<Expr>, op: BinOp, right: Box<Expr> },
    Unary { op: UnaryOp, expr: Box<Expr> },
    Call { func: Box<Expr>, args: Vec<Expr> },
    MethodCall { obj: Box<Expr>, method: String, args: Vec<Expr> },
    FieldAccess { obj: Box<Expr>, field: String },
    Index { obj: Box<Expr>, index: Box<Expr> },
    Array(Vec<Expr>),
    Tuple(Vec<Expr>),
    Struct { name: String, fields: Vec<(String, Expr)> },
    Lambda { params: Vec<Param>, body: Box<Expr> },
    Await(Box<Expr>),
    Borrow { expr: Box<Expr>, mutable: bool },
    Range { start: Option<Box<Expr>>, end: Option<Box<Expr>>, inclusive: bool },
    If { condition: Box<Expr>, then_expr: Box<Expr>, else_expr: Option<Box<Expr>> },
    Match { expr: Box<Expr>, arms: Vec<MatchArm> },
    Path(Vec<String>),
}
```

---

## IR Structure

```rust
pub enum IRNode {
    // Module level
    Module { name: String, items: Vec<IRNode> },

    // Declarations
    Function { name: String, params: Vec<IRParam>, ret_type: IRType,
               is_async: bool, body: Vec<IRNode> },
    Struct { name: String, fields: Vec<IRField> },
    Enum { name: String, variants: Vec<IRVariant> },

    // HTTP
    HttpHandler { method: HttpMethod, path: String, handler_name: String },
    ServerConfig { host: String, port: u16 },
    DbConfig { driver: String, config: HashMap<String, String> },

    // Statements
    Let { name: String, ty: IRType, value: Option<Box<IRNode>>, mutable: bool },
    Assign { target: Box<IRNode>, value: Box<IRNode> },
    If { cond: Box<IRNode>, then_block: Vec<IRNode>, else_block: Option<Vec<IRNode>> },
    Loop { body: Vec<IRNode> },
    ForRange { var: String, start: Box<IRNode>, end: Box<IRNode>, body: Vec<IRNode> },
    Match { expr: Box<IRNode>, arms: Vec<IRMatchArm> },
    Return(Option<Box<IRNode>>),
    Break,
    Continue,

    // Expressions
    Literal(IRLiteral),
    Ident(String),
    BinOp { op: BinOp, left: Box<IRNode>, right: Box<IRNode> },
    UnaryOp { op: UnaryOp, expr: Box<IRNode> },
    Call { func: String, args: Vec<IRNode> },
    MethodCall { receiver: Box<IRNode>, method: String, args: Vec<IRNode> },
    FieldAccess { obj: Box<IRNode>, field: String },
    Index { obj: Box<IRNode>, index: Box<IRNode> },
    Array { elements: Vec<IRNode> },
    StructInit { name: String, fields: Vec<(String, IRNode)> },
    Await(Box<IRNode>),
    Borrow { expr: Box<IRNode>, mutable: bool },
}
```

---

## CLI Commands

| Command | Description |
|---------|-------------|
| `mendes build <file.ms>` | Compile to native binary |
| `mendes check <file.ms>` | Type-check without compiling |
| `mendes lex <file.ms>` | Show tokens |
| `mendes parse <file.ms>` | Show AST |
| `mendes ir <file.ms>` | Show IR |
| `mendes emit <file.ms>` | Show generated Rust code |

### Build Options

```
mendes build <file.ms> [OPTIONS]

OPTIONS:
    -o, --output <name>     Output binary name
    -O, --optimize          Enable optimizations
    --release               Build in release mode
    --target <target>       Cross-compilation target
    --emit-rust             Also output Rust source
    --emit-c                Use C backend instead
```

---

## Type System Details

### Type Inference

Types can be inferred from context:

```mendes
let x = 42           // Inferred as int
let name = "Alice"   // Inferred as string
let items = [1,2,3]  // Inferred as [int]
```

### Type Checking Rules

1. **Binary operations** require compatible types
2. **Function calls** must match parameter types
3. **Return statements** must match declared return type
4. **Pattern matching** must be exhaustive
5. **Ownership** rules enforced at compile time

### Built-in Type Conversions

```mendes
let i: int = 42
let f: float = i.to_float()
let s: string = i.to_string()
let parsed: int = "123".parse_int()
```

---

## Ownership Rules

1. **Single Owner**: Each value has exactly one owner
2. **Move Semantics**: Assignment transfers ownership
3. **Borrow Rules**:
   - Multiple immutable borrows OR one mutable borrow
   - Borrows must not outlive owner
4. **No References Across Await**: References cannot cross await points

### Examples

```mendes
// Move
let a = vec![1, 2, 3]
let b = a              // a is moved to b
// print(a)            // Error: a was moved

// Borrow
let c = vec![1, 2, 3]
print(&c)              // Immutable borrow
print(c)               // c still valid

// Mutable borrow
let mut d = vec![1, 2, 3]
modify(&mut d)         // Mutable borrow
print(d)               // d still valid
```

---

## Error Handling

### Diagnostic Format

```
error[E0001]: type mismatch
 --> src/main.ms:10:15
   |
10 |     let x: int = "hello"
   |            ---   ^^^^^^^ expected int, found string
   |            |
   |            expected due to this type annotation
   |
   = help: consider using parse_int() to convert string to int
```

### Error Codes

| Code | Category |
|------|----------|
| E0001-E0099 | Type errors |
| E0100-E0199 | Ownership errors |
| E0200-E0299 | Syntax errors |
| E0300-E0399 | Resolution errors |
| E0400-E0499 | HTTP/API errors |

---

## Runtime Components

### HTTP Server

- Built on `hyper` and `tokio`
- Async request handling
- Route matching with path parameters
- Middleware support
- JSON serialization/deserialization

### Database

- Connection pooling via `sqlx`
- Supported drivers: PostgreSQL, MySQL, SQLite
- Async queries
- Prepared statements
- Transaction support

### WebSocket

- Full-duplex communication
- Message broadcasting
- Connection management
- Ping/pong handling

---

## Testing

### Run Tests

```bash
cargo test                    # All tests
cargo test --package mendes-lexer   # Lexer tests only
cargo test integration        # Integration tests only
```

### Test Coverage

- **Lexer**: Token generation, indentation handling, edge cases
- **Parser**: All AST nodes, error recovery
- **Semantic**: Type checking, ownership, symbol resolution
- **IR**: Lowering correctness
- **Codegen**: Output validity
- **Integration**: End-to-end compilation

---

## Build System

### Development Build

```bash
cargo build
cargo run -- build examples/hello.ms
```

### Release Build

```bash
cargo build --release
./target/release/mendes build examples/hello.ms --release
```

### Cross-Compilation

```bash
mendes build app.ms --target x86_64-unknown-linux-gnu
mendes build app.ms --target aarch64-apple-darwin
```

---

## Project Status

### Implemented

- [x] Complete lexer with indentation tracking
- [x] Recursive descent parser
- [x] Type system with inference
- [x] Ownership analysis
- [x] IR generation
- [x] Rust code generation
- [x] C code generation (secondary)
- [x] CLI with all commands
- [x] Integration tests (48 tests)
- [x] Comprehensive documentation

### Planned

- [ ] LSP server for IDE support
- [ ] Package manager
- [ ] REPL
- [ ] Debugging support
- [ ] Macro system

---

## Key Files Reference

| File | Purpose |
|------|---------|
| `crates/mendes-lexer/src/lexer.rs` | Tokenization logic |
| `crates/mendes-parser/src/parser.rs` | Parsing implementation |
| `crates/mendes-parser/src/ast.rs` | AST definitions |
| `crates/mendes-semantic/src/analyzer.rs` | Type checking |
| `crates/mendes-semantic/src/ownership.rs` | Ownership rules |
| `crates/mendes-ir/src/lower.rs` | AST to IR conversion |
| `crates/mendes-codegen/src/rust_gen.rs` | Rust code generation |
| `crates/mendes-cli/src/main.rs` | CLI entry point |
| `crates/mendes-runtime/src/lib.rs` | Runtime library |

---

## Common Patterns

### Creating a New HTTP Endpoint

```mendes
api GET /items/{id}:
    let item = await db.query_one("SELECT * FROM items WHERE id = $1", [id])
    match item:
        Some(i):
            return json(i)
        None:
            return status(404).json({ "error": "Not found" })
```

### Defining a Struct with Methods

```mendes
struct User:
    id: int
    name: string
    email: string

impl User:
    fn new(name: string, email: string) -> User:
        User { id: 0, name, email }

    fn display(&self) -> string:
        "{self.name} <{self.email}>"
```

### Error Handling Pattern

```mendes
fn process_data(input: string) -> Result<Data, Error>:
    let parsed = input.parse_json()?
    let validated = validate(parsed)?
    return Ok(transform(validated))
```

---

## Development Guidelines

1. **All code must pass `cargo check`** before committing
2. **Run `cargo test`** to ensure no regressions
3. **Follow Rust naming conventions** in generated code
4. **Maintain backwards compatibility** in the language syntax
5. **Document new features** in the appropriate docs file

---

*This document serves as context for continuing development of the Mendes programming language.*
