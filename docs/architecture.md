# Arquitetura do Compilador Mendes

> **Versao**: 0.1.0
> **Autor**: Guilherme Mendes

Este documento descreve a arquitetura interna do compilador Mendes, suas fases, estruturas de dados e decisoes de design.

---

## Indice

1. [Visao Geral](#1-visao-geral)
2. [Pipeline de Compilacao](#2-pipeline-de-compilacao)
3. [Lexer (mendes-lexer)](#3-lexer-mendes-lexer)
4. [Parser (mendes-parser)](#4-parser-mendes-parser)
5. [Analise Semantica (mendes-semantic)](#5-analise-semantica-mendes-semantic)
6. [IR (mendes-ir)](#6-ir-mendes-ir)
7. [Codegen (mendes-codegen)](#7-codegen-mendes-codegen)
8. [Runtime (mendes-runtime)](#8-runtime-mendes-runtime)
9. [Sistema de Erros (mendes-error)](#9-sistema-de-erros-mendes-error)
10. [CLI (mendes-cli)](#10-cli-mendes-cli)
11. [Decisoes de Design](#11-decisoes-de-design)
12. [Fluxo de Dados](#12-fluxo-de-dados)

---

## 1. Visao Geral

### Estrutura do Projeto

```
linguagem-mendes/
├── Cargo.toml                 # Workspace Rust
├── crates/
│   ├── mendes-error/          # Diagnosticos e spans
│   ├── mendes-lexer/          # Tokenizacao
│   ├── mendes-parser/         # Parser e AST
│   ├── mendes-semantic/       # Analise semantica
│   ├── mendes-ir/             # Representacao intermediaria
│   ├── mendes-codegen/        # Geracao de codigo
│   ├── mendes-runtime/        # Runtime de execucao
│   ├── mendes-cli/            # Interface de linha de comando
│   └── mendes-tests/          # Testes de integracao
├── docs/                      # Documentacao
└── examples/                  # Programas de exemplo
```

### Dependencias entre Crates

```
                    ┌──────────────┐
                    │  mendes-cli  │
                    └──────┬───────┘
                           │
         ┌─────────────────┼─────────────────┐
         │                 │                 │
         ▼                 ▼                 ▼
┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│mendes-codegen│  │  mendes-ir   │  │mendes-runtime│
└──────┬───────┘  └──────┬───────┘  └──────────────┘
       │                 │
       │    ┌────────────┘
       │    │
       ▼    ▼
┌──────────────┐
│mendes-semantic│
└──────┬───────┘
       │
       ▼
┌──────────────┐
│ mendes-parser │
└──────┬───────┘
       │
       ▼
┌──────────────┐
│ mendes-lexer │
└──────┬───────┘
       │
       ▼
┌──────────────┐
│ mendes-error │
└──────────────┘
```

---

## 2. Pipeline de Compilacao

### Fases

```
┌─────────┐    ┌────────┐    ┌──────────┐    ┌────┐    ┌─────────┐    ┌──────────┐
│ Source  │───▶│ Lexer  │───▶│  Parser  │───▶│ AST│───▶│Semantic │───▶│    IR    │
│  Code   │    │        │    │          │    │    │    │ Analysis│    │          │
└─────────┘    └────────┘    └──────────┘    └────┘    └─────────┘    └──────────┘
                   │              │                         │              │
                   │              │                         │              │
                   ▼              ▼                         ▼              ▼
               [Tokens]       [AST]                   [Typed AST]        [IR]
                                                                          │
                                                                          ▼
                                                                    ┌──────────┐
                                                                    │ Codegen  │
                                                                    └────┬─────┘
                                                                         │
                                            ┌────────────────────────────┼────────────────────────────┐
                                            │                            │                            │
                                            ▼                            ▼                            ▼
                                      ┌──────────┐                ┌──────────┐                ┌──────────┐
                                      │   Rust   │                │    C     │                │   LLVM   │
                                      │  Backend │                │ Backend  │                │ Backend  │
                                      └────┬─────┘                └────┬─────┘                └────┬─────┘
                                           │                           │                           │
                                           ▼                           ▼                           ▼
                                      [Rust Code]                 [C Code]                   [LLVM IR]
                                           │                           │                           │
                                           ▼                           ▼                           ▼
                                        Cargo                        GCC                        LLVM
                                           │                           │                           │
                                           └───────────────────────────┼───────────────────────────┘
                                                                       │
                                                                       ▼
                                                               [Native Binary]
```

### Tempo de Cada Fase (Tipico)

| Fase | Tempo | % do Total |
|------|-------|------------|
| Lexer | ~5ms | 5% |
| Parser | ~10ms | 10% |
| Semantic | ~15ms | 15% |
| IR | ~5ms | 5% |
| Codegen | ~10ms | 10% |
| Cargo Build | ~55ms | 55% |

---

## 3. Lexer (mendes-lexer)

### Responsabilidades

1. Converter codigo fonte em tokens
2. Rastrear posicao (linha/coluna)
3. Gerenciar indentacao significativa
4. Reportar erros lexicos

### Estruturas Principais

```rust
// Token com posicao
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

// Tipos de token
pub enum TokenKind {
    // Keywords
    Let, Mut, Fn, Return, If, Else, ...

    // Literals
    IntLit(i64),
    FloatLit(f64),
    StringLit(String),

    // Operators
    Plus, Minus, Star, Slash, ...

    // Delimiters
    Colon, Comma, Dot, ...

    // Indentation
    Newline, Indent, Dedent,

    // Special
    Eof, Error(String),
}
```

### Algoritmo de Indentacao

```rust
// Stack de niveis de indentacao
indent_stack: Vec<usize> = vec![0];

fn process_newline(&mut self) {
    let spaces = self.count_leading_spaces();

    if spaces > *self.indent_stack.last() {
        // Aumento de indentacao
        self.indent_stack.push(spaces);
        self.emit(Indent);
    } else {
        // Reducao de indentacao
        while spaces < *self.indent_stack.last() {
            self.indent_stack.pop();
            self.emit(Dedent);
        }
    }
}
```

### Exemplo de Tokenizacao

```mendes
# Input
fn add(a: int, b: int) -> int:
    return a + b
```

```
# Output
Fn("fn") @ 1:1
Ident("add") @ 1:4
LParen @ 1:7
Ident("a") @ 1:8
Colon @ 1:9
IntType @ 1:11
Comma @ 1:14
Ident("b") @ 1:16
Colon @ 1:17
IntType @ 1:19
RParen @ 1:22
Arrow @ 1:24
IntType @ 1:27
Colon @ 1:30
Newline @ 1:31
Indent @ 2:1
Return @ 2:5
Ident("a") @ 2:12
Plus @ 2:14
Ident("b") @ 2:16
Newline @ 2:17
Dedent @ 3:1
Eof @ 3:1
```

---

## 4. Parser (mendes-parser)

### Responsabilidades

1. Construir AST a partir dos tokens
2. Validar sintaxe
3. Reportar erros de parse

### Tipo de Parser

**Recursive Descent com Pratt Parsing para expressoes**

- Cada regra gramatical e uma funcao
- Precedencia de operadores via Pratt parsing
- Lookahead de 1 token (LL(1))

### AST Principal

```rust
pub struct Program {
    pub statements: Vec<Stmt>,
}

pub enum Stmt {
    Let { name: String, ty: Option<Type>, value: Expr, mutable: bool, span: Span },
    Fn(FnDecl),
    Struct(StructDecl),
    Enum(EnumDecl),
    Api(ApiDecl),
    Server(ServerDecl),
    If { condition: Expr, then_block: Vec<Stmt>, else_block: Option<Vec<Stmt>>, span: Span },
    For { var: String, iter: Expr, body: Vec<Stmt>, span: Span },
    While { condition: Expr, body: Vec<Stmt>, span: Span },
    Return { value: Option<Expr>, span: Span },
    Expr(Expr),
    // ...
}

pub enum Expr {
    IntLit(i64, Span),
    FloatLit(f64, Span),
    StringLit(String, Span),
    BoolLit(bool, Span),
    Ident(String, Span),
    Binary { left: Box<Expr>, op: BinOp, right: Box<Expr>, span: Span },
    Unary { op: UnaryOp, expr: Box<Expr>, span: Span },
    Call { func: Box<Expr>, args: Vec<Expr>, span: Span },
    // ...
}
```

### Pratt Parsing

```rust
fn parse_expression(&mut self, min_precedence: u8) -> Expr {
    let mut left = self.parse_prefix();

    while self.current_precedence() > min_precedence {
        left = self.parse_infix(left);
    }

    left
}

fn precedence(op: &BinOp) -> u8 {
    match op {
        BinOp::Assign => 1,
        BinOp::Or => 2,
        BinOp::And => 3,
        BinOp::Eq | BinOp::Ne => 5,
        BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => 5,
        BinOp::Add | BinOp::Sub => 6,
        BinOp::Mul | BinOp::Div | BinOp::Mod => 7,
    }
}
```

---

## 5. Analise Semantica (mendes-semantic)

### Responsabilidades

1. Type checking
2. Resolucao de nomes
3. Ownership checking
4. Validacao de referencias

### Type Checker

```rust
pub struct SemanticContext {
    // Tabela de simbolos hierarquica
    scopes: Vec<Scope>,

    // Tipos definidos
    types: HashMap<String, TypeDef>,

    // Funcoes definidas
    functions: HashMap<String, FnSignature>,
}

pub struct Scope {
    variables: HashMap<String, VariableInfo>,
    parent: Option<usize>,
}

pub struct VariableInfo {
    ty: Type,
    mutable: bool,
    initialized: bool,
    moved: bool,
    borrows: Vec<BorrowInfo>,
}
```

### Algoritmo de Inferencia

```rust
fn infer_type(&mut self, expr: &Expr) -> Result<Type, TypeError> {
    match expr {
        Expr::IntLit(_, _) => Ok(Type::Int),
        Expr::FloatLit(_, _) => Ok(Type::Float),
        Expr::StringLit(_, _) => Ok(Type::String),
        Expr::BoolLit(_, _) => Ok(Type::Bool),

        Expr::Ident(name, span) => {
            self.lookup_variable(name, *span)
                .map(|v| v.ty.clone())
        }

        Expr::Binary { left, op, right, .. } => {
            let left_ty = self.infer_type(left)?;
            let right_ty = self.infer_type(right)?;
            self.check_binary_op(&left_ty, op, &right_ty)
        }

        Expr::Call { func, args, .. } => {
            let fn_ty = self.infer_type(func)?;
            self.check_call(&fn_ty, args)
        }

        // ...
    }
}
```

### Ownership Checker

```rust
pub struct OwnershipChecker {
    // Variaveis movidas
    moved: HashSet<String>,

    // Emprestimos ativos
    borrows: Vec<BorrowInfo>,
}

pub struct BorrowInfo {
    variable: String,
    mutable: bool,
    span: Span,
}

fn check_borrow(&mut self, var: &str, mutable: bool, span: Span) -> Result<(), Error> {
    // Verifica se foi movido
    if self.moved.contains(var) {
        return Err(Error::UseAfterMove(var.to_string(), span));
    }

    // Verifica conflitos de emprestimo
    for borrow in &self.borrows {
        if borrow.variable == var {
            if mutable || borrow.mutable {
                return Err(Error::BorrowConflict(var.to_string(), span));
            }
        }
    }

    self.borrows.push(BorrowInfo { variable: var.to_string(), mutable, span });
    Ok(())
}
```

---

## 6. IR (mendes-ir)

### Responsabilidades

1. Representacao intermediaria de baixo nivel
2. Otimizacoes independentes de target
3. Facilitar geracao de codigo

### Estrutura do IR

```rust
pub struct Module {
    pub name: String,
    pub structs: Vec<StructDef>,
    pub functions: Vec<Function>,
    pub routes: Vec<HttpRoute>,
    pub string_table: Vec<String>,
}

pub struct Function {
    pub name: String,
    pub params: Vec<(String, IrType)>,
    pub return_type: IrType,
    pub is_async: bool,
    pub blocks: Vec<BasicBlock>,
}

pub struct BasicBlock {
    pub label: String,
    pub instructions: Vec<Instruction>,
    pub terminator: Terminator,
}

pub enum Instruction {
    // Alocacao
    Alloca { dest: String, ty: IrType },

    // Carrega/Armazena
    Load { dest: String, ptr: String },
    Store { ptr: String, value: String },

    // Aritmetica
    BinOp { dest: String, op: BinOp, left: String, right: String },

    // Chamadas
    Call { dest: Option<String>, func: String, args: Vec<String> },

    // Campos
    GetField { dest: String, ptr: String, struct_name: String, field_index: usize },
    SetField { ptr: String, struct_name: String, field_index: usize, value: String },
}

pub enum Terminator {
    Return(Option<String>),
    Branch(String),
    CondBranch { cond: String, then_block: String, else_block: String },
}
```

### Lowering AST -> IR

```rust
pub fn lower_program(program: &Program) -> Module {
    let mut lowerer = Lowerer::new();

    for stmt in &program.statements {
        lowerer.lower_statement(stmt);
    }

    lowerer.module
}

impl Lowerer {
    fn lower_statement(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Fn(f) => self.lower_function(f),
            Stmt::Struct(s) => self.lower_struct(s),
            Stmt::Api(a) => self.lower_api(a),
            // ...
        }
    }

    fn lower_function(&mut self, f: &FnDecl) {
        let mut func = Function::new(&f.name);

        for param in &f.params {
            func.add_param(&param.name, self.lower_type(&param.ty));
        }

        let entry = func.create_block("entry");
        self.current_block = entry;

        for stmt in &f.body {
            self.lower_body_statement(stmt);
        }

        self.module.functions.push(func);
    }
}
```

---

## 7. Codegen (mendes-codegen)

### Backends Disponiveis

| Backend | Status | Descricao |
|---------|--------|-----------|
| Rust | Completo | Gera codigo Rust, compila via Cargo |
| C | Basico | Gera codigo C, compila via GCC |
| LLVM | Planejado | Gera LLVM IR diretamente |

### Trait CodeGen

```rust
pub trait CodeGen {
    type Output;
    fn generate(&self, module: &Module) -> Self::Output;
}

pub struct RustBackend { ... }
pub struct CBackend { ... }

impl CodeGen for RustBackend {
    type Output = String;

    fn generate(&self, module: &Module) -> String {
        let mut code = String::new();

        // Imports
        code.push_str("use mendes_runtime::*;\n\n");

        // Structs
        for s in &module.structs {
            code.push_str(&self.generate_struct(s));
        }

        // Functions
        for f in &module.functions {
            code.push_str(&self.generate_function(f));
        }

        // Main com router
        code.push_str(&self.generate_main(module));

        code
    }
}
```

### Rust Backend

```rust
impl RustBackend {
    fn generate_function(&self, func: &Function) -> String {
        let mut code = String::new();

        // Assinatura
        let async_kw = if func.is_async { "async " } else { "" };
        let params = func.params.iter()
            .map(|(n, t)| format!("{}: {}", n, self.rust_type(t)))
            .collect::<Vec<_>>()
            .join(", ");
        let ret = self.rust_type(&func.return_type);

        code.push_str(&format!(
            "{}fn {}({}) -> {} {{\n",
            async_kw, func.name, params, ret
        ));

        // Corpo
        for block in &func.blocks {
            for inst in &block.instructions {
                code.push_str(&self.generate_instruction(inst));
            }
            code.push_str(&self.generate_terminator(&block.terminator));
        }

        code.push_str("}\n\n");
        code
    }
}
```

---

## 8. Runtime (mendes-runtime)

### Componentes

```
mendes-runtime/
├── src/
│   ├── lib.rs           # Exports publicos
│   ├── http.rs          # Server HTTP (hyper)
│   ├── router.rs        # Roteamento de requests
│   ├── database.rs      # Pool de conexoes (sqlx)
│   ├── websocket.rs     # WebSocket (tungstenite)
│   ├── middleware.rs    # Sistema de middleware
│   ├── types.rs         # Tipos utilitarios
│   └── error.rs         # Tipos de erro
```

### HTTP Server

```rust
pub struct Server {
    addr: SocketAddr,
    router: Option<Router>,
}

impl Server {
    pub fn new(addr: &str) -> Self { ... }

    pub fn router(mut self, router: Router) -> Self { ... }

    pub async fn run(self) -> Result<()> {
        let make_svc = make_service_fn(|_| {
            let router = self.router.clone();
            async move {
                Ok::<_, Infallible>(service_fn(move |req| {
                    handle_request(req, router.clone())
                }))
            }
        });

        let server = HyperServer::bind(&self.addr).serve(make_svc);
        server.await?;
        Ok(())
    }
}
```

### Router

```rust
pub struct Router {
    routes: Vec<Route>,
    middlewares: Vec<Box<dyn Middleware>>,
}

struct Route {
    method: HttpMethod,
    pattern: PathPattern,
    handler: Box<dyn Handler>,
}

impl Router {
    pub fn get<H>(&mut self, path: &str, handler: H)
    where
        H: Handler + 'static
    {
        self.routes.push(Route {
            method: HttpMethod::Get,
            pattern: PathPattern::parse(path),
            handler: Box::new(handler),
        });
    }

    pub async fn route(&self, req: Request) -> Response {
        for route in &self.routes {
            if route.matches(&req) {
                return route.handler.handle(req).await;
            }
        }
        Response::not_found("Route not found")
    }
}
```

---

## 9. Sistema de Erros (mendes-error)

### Estruturas

```rust
pub struct Diagnostic {
    pub level: Level,
    pub message: String,
    pub code: Option<String>,
    pub span: Span,
    pub labels: Vec<Label>,
    pub notes: Vec<String>,
    pub suggestions: Vec<Suggestion>,
}

pub enum Level {
    Error,
    Warning,
    Note,
    Help,
}

pub struct Span {
    pub file_id: usize,
    pub start: Position,
    pub end: Position,
}

pub struct Position {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}
```

### Renderizacao

```rust
impl DiagnosticRenderer {
    pub fn render(&self, diag: &Diagnostic) -> String {
        let mut output = String::new();

        // Header
        output.push_str(&format!(
            "{}[{}]: {}\n",
            diag.level,
            diag.code.as_deref().unwrap_or("E0000"),
            diag.message
        ));

        // Location
        let file = self.cache.get_file(diag.span.file_id);
        output.push_str(&format!(
            " --> {}:{}:{}\n",
            file.name, diag.span.start.line, diag.span.start.column
        ));

        // Code snippet
        output.push_str(&self.render_snippet(diag));

        // Notes and suggestions
        for note in &diag.notes {
            output.push_str(&format!("  = note: {}\n", note));
        }
        for suggestion in &diag.suggestions {
            output.push_str(&format!("  = help: {}\n", suggestion.message));
        }

        output
    }
}
```

### Exemplo de Saida

```
error[E0205]: use of moved value
 --> src/main.ms:10:12
   |
8  |     let user = get_user()
   |         ---- value defined here
9  |     consume(user)
   |             ---- value moved here
10 |     print(user.name)
   |           ^^^^ value used after move
   |
   = note: move occurs because `user` has type `User`, which does not implement `Copy`
   = help: consider borrowing the value: `&user`
```

---

## 10. CLI (mendes-cli)

### Comandos

```rust
#[derive(Subcommand)]
enum Commands {
    Build { input: PathBuf, output: Option<PathBuf>, backend: Backend, release: bool },
    Check { input: PathBuf },
    Run { input: PathBuf },
    Lex { input: PathBuf },
    Parse { input: PathBuf },
    Ir { input: PathBuf },
    Emit { input: PathBuf, output: Option<PathBuf> },
    EmitRust { input: PathBuf, output: Option<PathBuf> },
}
```

### Fluxo do Build

```rust
fn build(input: &Path, output: &str, backend: Backend, release: bool) {
    // 1. Ler arquivo fonte
    let source = fs::read_to_string(input)?;

    // 2. Lexer
    let mut lexer = Lexer::new(&source, 0);
    let tokens = lexer.tokenize();

    // 3. Parser
    let (program, parse_diags) = parse(tokens);
    if parse_diags.has_errors() {
        report_errors(parse_diags);
        exit(1);
    }

    // 4. Semantic
    let mut ctx = SemanticContext::new();
    let semantic_diags = analyze(&program, &mut ctx);
    if semantic_diags.has_errors() {
        report_errors(semantic_diags);
        exit(1);
    }

    // 5. IR
    let ir_module = lower_program(&program);

    // 6. Codegen
    match backend {
        Backend::Rust => build_with_rust(ir_module, output, release),
        Backend::C => build_with_c(ir_module, output),
    }
}
```

---

## 11. Decisoes de Design

### Por que Indentacao Significativa?

**Pros:**
- Codigo mais limpo e legivel
- Forca boas praticas de formatacao
- Menos "noise" sintatico

**Cons:**
- Pode causar erros confusos
- Copy-paste de codigo pode quebrar

**Decisao:** Adotado por alinhar com o publico-alvo (desenvolvedores Python/backend).

### Por que Rust como Backend Principal?

**Pros:**
- Acesso ao ecossistema Cargo/crates.io
- Seguranca de memoria garantida
- Performance excelente
- Async nativo com Tokio

**Cons:**
- Tempo de compilacao
- Dependencia do Rust instalado

**Decisao:** Beneficios superam custos para projetos de backend.

### Por que Ownership Simplificado?

O sistema de ownership de Mendes e uma versao simplificada do Rust:

- **Sem lifetimes explicitos**: Inferidos automaticamente
- **Sem Box/Rc/Arc explicitos**: Gerenciados pelo compilador
- **Sem unsafe**: Nao exposto ao usuario

**Razao:** Balanco entre seguranca e usabilidade.

---

## 12. Fluxo de Dados

### Compilacao de API

```
# Input Mendes
api GET /users/{id:int}:
    return User
    return find_user(id)

# Apos Lexer
[Api, Get, Path("/users/{id:int}"), Colon, Newline, Indent, ...]

# Apos Parser (AST)
ApiDecl {
    method: Get,
    path: "/users/{id:int}",
    return_type: Some(Type::Named("User")),
    handler: [ReturnStmt { value: Call { func: "find_user", args: [Ident("id")] } }]
}

# Apos IR
HttpRoute {
    method: Get,
    pattern: "/users/:id",
    handler: "handle_users_id_get"
}
Function {
    name: "handle_users_id_get",
    params: [("id", Int)],
    return_type: User,
    blocks: [...]
}

# Apos Codegen (Rust)
async fn handle_users_id_get(req: Request) -> Response {
    let id: i64 = req.param("id").unwrap().parse().unwrap();
    let result = find_user(id);
    Response::ok(result)
}

router.get("/users/:id", |req| async move {
    handle_users_id_get(req).await
});
```

---

## Apendice: Metricas do Projeto

### Linhas de Codigo (Aproximado)

| Crate | LOC | Descricao |
|-------|-----|-----------|
| mendes-error | ~400 | Sistema de diagnosticos |
| mendes-lexer | ~600 | Tokenizacao |
| mendes-parser | ~1500 | Parser e AST |
| mendes-semantic | ~800 | Type checker |
| mendes-ir | ~600 | IR e lowering |
| mendes-codegen | ~1200 | Backends de codegen |
| mendes-runtime | ~1000 | Runtime HTTP/DB/WS |
| mendes-cli | ~900 | Interface de comando |
| **Total** | **~7000** | |

### Cobertura de Testes

| Tipo | Quantidade |
|------|------------|
| Testes unitarios | ~50 |
| Testes de integracao | ~48 |
| Exemplos testados | ~16 |

---

<p align="center">
  <strong>Mendes Compiler Architecture v0.1.0</strong>
</p>
