//! AST - Abstract Syntax Tree for the Mendes language

use mendes_error::Span;

/// Complete program (source file)
#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

/// Statements (declarations)
#[derive(Debug, Clone)]
pub enum Stmt {
    /// `import "path/to/file.ms"` or `import module_name`
    Import {
        path: String,
        alias: Option<String>,
        span: Span,
    },

    /// `from module import item1, item2` or `from module import *`
    FromImport {
        module: String,
        items: ImportItems,
        span: Span,
    },

    /// `let x: int = 10` or `let mut x = 10`
    Let {
        name: String,
        ty: Option<Type>,
        value: Expr,
        mutable: bool,
        span: Span,
    },

    /// Function declaration
    Fn(FnDecl),

    /// Struct declaration
    Struct(StructDecl),

    /// Enum declaration
    Enum(EnumDecl),

    /// Trait declaration
    Trait(TraitDecl),

    /// Trait implementation: `impl TraitName for TypeName:`
    ImplTrait(ImplTraitDecl),

    /// Type alias: `type UserId = int`
    TypeAlias {
        name: String,
        ty: Type,
        span: Span,
    },

    /// HTTP API declaration
    Api(ApiDecl),

    /// WebSocket endpoint declaration
    WebSocket(WsDecl),

    /// Server declaration
    Server(ServerDecl),

    /// Middleware declaration
    Middleware(MiddlewareDecl),

    /// Database connection declaration
    Db(DbDecl),

    /// `if cond: ... else: ...`
    If {
        condition: Expr,
        then_block: Vec<Stmt>,
        else_block: Option<Vec<Stmt>>,
        span: Span,
    },

    /// `for x in items: ...`
    For {
        var: String,
        iter: Expr,
        body: Vec<Stmt>,
        span: Span,
    },

    /// `while cond: ...`
    While {
        condition: Expr,
        body: Vec<Stmt>,
        span: Span,
    },

    /// `return expr`
    Return {
        value: Option<Expr>,
        span: Span,
    },

    /// `break` - exit loop
    Break { span: Span },

    /// `continue` - next iteration
    Continue { span: Span },

    /// Expression as statement
    Expr(Expr),
}

/// Function declaration
#[derive(Debug, Clone)]
pub struct FnDecl {
    pub name: String,
    pub generic_params: Vec<GenericParam>,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub is_async: bool,
    pub is_pub: bool,
    pub body: Vec<Stmt>,
    pub span: Span,
}

/// Generic type parameter: `T` or `T: Trait`
#[derive(Debug, Clone)]
pub struct GenericParam {
    pub name: String,
    pub bounds: Vec<String>,
    pub span: Span,
}

/// Function parameter
#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub ty: Type,
    pub span: Span,
}

/// Struct declaration
#[derive(Debug, Clone)]
pub struct StructDecl {
    pub name: String,
    pub generic_params: Vec<GenericParam>,
    pub fields: Vec<Field>,
    pub methods: Vec<MethodDecl>,
    pub is_copy: bool,
    pub span: Span,
}

/// Method declaration (inside a struct)
#[derive(Debug, Clone)]
pub struct MethodDecl {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub is_async: bool,
    pub is_pub: bool,
    /// Whether this method takes &self, &mut self, or self
    pub receiver: MethodReceiver,
    pub body: Vec<Stmt>,
    pub span: Span,
}

/// Method receiver type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MethodReceiver {
    /// &self - immutable borrow
    Ref,
    /// &mut self - mutable borrow
    MutRef,
    /// self - takes ownership
    Value,
}

/// Struct field
#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub ty: Type,
    pub span: Span,
}

/// Enum declaration
#[derive(Debug, Clone)]
pub struct EnumDecl {
    pub name: String,
    pub variants: Vec<EnumVariant>,
    pub span: Span,
}

/// Enum variant
#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: String,
    /// Optional associated data: Unit, Tuple, or Struct
    pub data: EnumVariantData,
    pub span: Span,
}

/// Enum variant associated data
#[derive(Debug, Clone)]
pub enum EnumVariantData {
    /// No data: `None` in `Option`
    Unit,
    /// Tuple-style: `Some(T)` in `Option<T>`
    Tuple(Vec<Type>),
    /// Struct-style: `Point { x: int, y: int }`
    Struct(Vec<Field>),
}

/// Trait declaration: `trait Name:`
#[derive(Debug, Clone)]
pub struct TraitDecl {
    pub name: String,
    pub generic_params: Vec<GenericParam>,
    pub methods: Vec<TraitMethod>,
    pub span: Span,
}

/// Trait method signature (no body)
#[derive(Debug, Clone)]
pub struct TraitMethod {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub is_async: bool,
    pub receiver: MethodReceiver,
    pub span: Span,
}

/// Trait implementation: `impl TraitName for TypeName:`
#[derive(Debug, Clone)]
pub struct ImplTraitDecl {
    pub trait_name: String,
    pub type_name: String,
    pub generic_params: Vec<GenericParam>,
    pub methods: Vec<MethodDecl>,
    pub span: Span,
}

/// HTTP API declaration
#[derive(Debug, Clone)]
pub struct ApiDecl {
    pub method: HttpMethod,
    pub path: String,
    pub is_async: bool,
    pub middlewares: Vec<String>,
    pub body_type: Option<Type>,
    pub return_type: Option<Type>,
    pub handler: Vec<Stmt>,
    pub span: Span,
}

/// HTTP methods
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

/// WebSocket endpoint declaration
#[derive(Debug, Clone)]
pub struct WsDecl {
    pub path: String,
    pub middlewares: Vec<String>,
    pub on_connect: Option<Vec<Stmt>>,
    pub on_message: Option<Vec<Stmt>>,
    pub on_disconnect: Option<Vec<Stmt>>,
    pub span: Span,
}

/// WebSocket event types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WsEvent {
    Connect,
    Message,
    Disconnect,
}

/// Server declaration
#[derive(Debug, Clone)]
pub struct ServerDecl {
    pub host: String,
    pub port: u16,
    pub span: Span,
}

/// Middleware declaration
#[derive(Debug, Clone)]
pub struct MiddlewareDecl {
    pub name: String,
    pub body: Vec<Stmt>,
    pub span: Span,
}

/// Database connection declaration
#[derive(Debug, Clone)]
pub struct DbDecl {
    pub db_type: DbType,
    pub name: String,
    pub url: String,
    pub pool_size: u32,
    pub span: Span,
}

/// Database types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbType {
    Postgres,
    Mysql,
    Sqlite,
}

/// Types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    /// Primitive types
    Int,
    Float,
    Bool,
    String,

    /// User-defined type
    Named(std::string::String),

    /// Generic type: Result<T, E>, Option<T>
    Generic {
        name: std::string::String,
        args: Vec<Type>,
    },

    /// Reference: &T
    Ref(Box<Type>),

    /// Mutable reference: &mut T
    MutRef(Box<Type>),

    /// Array: [T]
    Array(Box<Type>),

    /// Tuple: (int, string, bool)
    Tuple(Vec<Type>),

    /// Function type: fn(int, int) -> int
    Function {
        params: Vec<Type>,
        return_type: Box<Type>,
    },
}

/// Expressions
#[derive(Debug, Clone)]
pub enum Expr {
    /// Integer literal
    IntLit(i64, Span),

    /// Float literal
    FloatLit(f64, Span),

    /// String literal
    StringLit(String, Span),

    /// Boolean literal
    BoolLit(bool, Span),

    /// None
    None(Span),

    /// Identifier
    Ident(String, Span),

    /// Binary operation
    Binary {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
        span: Span,
    },

    /// Unary operation
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
        span: Span,
    },

    /// Function call
    Call {
        func: Box<Expr>,
        args: Vec<Expr>,
        span: Span,
    },

    /// Method call: obj.method(args)
    MethodCall {
        object: Box<Expr>,
        method: String,
        args: Vec<Expr>,
        span: Span,
    },

    /// Field access
    FieldAccess {
        object: Box<Expr>,
        field: String,
        span: Span,
    },

    /// Index access
    Index {
        object: Box<Expr>,
        index: Box<Expr>,
        span: Span,
    },

    /// Await
    Await {
        expr: Box<Expr>,
        span: Span,
    },

    /// Borrow: &expr
    Borrow {
        expr: Box<Expr>,
        mutable: bool,
        span: Span,
    },

    /// Ok(expr)
    Ok(Box<Expr>, Span),

    /// Err(expr)
    Err(Box<Expr>, Span),

    /// Some(expr)
    Some(Box<Expr>, Span),

    /// Struct literal: User { name: "Ana", age: 30 }
    StructLit {
        name: String,
        fields: Vec<(String, Expr)>,
        span: Span,
    },

    /// Array literal: [1, 2, 3]
    ArrayLit(Vec<Expr>, Span),

    /// Match expression: `match expr: ...`
    Match {
        expr: Box<Expr>,
        arms: Vec<MatchArm>,
        span: Span,
    },

    /// Try expression: `expr?` - propagates errors
    Try {
        expr: Box<Expr>,
        span: Span,
    },

    /// Closure expression: `|x, y| x + y` or `|x: int| -> int: x * 2`
    Closure {
        params: Vec<ClosureParam>,
        return_type: Option<Type>,
        body: ClosureBody,
        span: Span,
    },

    /// String interpolation: f"hello {name}!"
    StringInterpolation {
        parts: Vec<StringPart>,
        span: Span,
    },

    /// Tuple expression: (1, "hello", true)
    Tuple {
        elements: Vec<Expr>,
        span: Span,
    },

    /// Range expression: 0..10 or 0..=10
    Range {
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
        inclusive: bool,
        span: Span,
    },
}

/// Closure parameter (can have optional type)
#[derive(Debug, Clone)]
pub struct ClosureParam {
    pub name: String,
    pub ty: Option<Type>,
    pub span: Span,
}

/// Closure body - either a single expression or a block
#[derive(Debug, Clone)]
pub enum ClosureBody {
    /// Single expression: `|x| x + 1`
    Expr(Box<Expr>),
    /// Block: `|x|: ...`
    Block(Vec<Stmt>),
}

/// Part of an interpolated string
#[derive(Debug, Clone)]
pub enum StringPart {
    /// Literal string part
    Literal(String),
    /// Expression to be formatted
    Expr(Expr),
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,

    // Comparison
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,

    // Logical
    And,
    Or,

    // Assignment
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
}

/// Match arm (a single case in a match expression)
#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expr>,
    pub body: Vec<Stmt>,
    pub span: Span,
}

/// Pattern for pattern matching
#[derive(Debug, Clone)]
pub enum Pattern {
    /// Wildcard pattern: `_`
    Wildcard(Span),

    /// Literal pattern: `42`, `"hello"`, `true`
    Literal(Expr),

    /// Identifier pattern (binds value): `x`, `name`
    Ident {
        name: String,
        mutable: bool,
        span: Span,
    },

    /// Tuple pattern: `(x, y, z)`
    Tuple(Vec<Pattern>, Span),

    /// Struct pattern: `Point { x, y }` or `Point { x: a, y: b }`
    Struct {
        name: String,
        fields: Vec<(String, Option<Pattern>)>,
        span: Span,
    },

    /// Enum variant pattern: `Some(x)`, `Color::Red`, `Message::Move { x, y }`
    Variant {
        enum_name: Option<String>,
        variant: String,
        data: VariantPatternData,
        span: Span,
    },

    /// Or pattern: `A | B | C`
    Or(Vec<Pattern>, Span),

    /// Range pattern: `1..10` or `1..=10`
    Range {
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
        inclusive: bool,
        span: Span,
    },
}

/// Data in a variant pattern
#[derive(Debug, Clone)]
pub enum VariantPatternData {
    /// No data: `None`
    Unit,
    /// Tuple data: `Some(x)`, `Ok(value)`
    Tuple(Vec<Pattern>),
    /// Struct data: `Move { x, y }`
    Struct(Vec<(String, Option<Pattern>)>),
}

/// Items being imported from a module
#[derive(Debug, Clone)]
pub enum ImportItems {
    /// Import all public items: `from module import *`
    All,
    /// Import specific items: `from module import foo, bar`
    Names(Vec<ImportItem>),
}

/// Single imported item with optional alias
#[derive(Debug, Clone)]
pub struct ImportItem {
    pub name: String,
    pub alias: Option<String>,
    pub span: Span,
}
