//! Tokens for the Mendes language
//!
//! Defines all token types that the lexer can produce.

use mendes_error::span::Span;
use std::fmt;

/// All token types for the Mendes language
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // =========================================
    // Keywords - Control flow
    // =========================================
    /// `let` - variable declaration
    Let,
    /// `mut` - mutability modifier
    Mut,
    /// `fn` - function declaration
    Fn,
    /// `return` - function return
    Return,
    /// `if` - conditional
    If,
    /// `else` - if alternative
    Else,
    /// `for` - for loop
    For,
    /// `in` - used in for loops
    In,
    /// `while` - while loop
    While,
    /// `break` - exit loop
    Break,
    /// `continue` - next iteration
    Continue,
    /// `match` - pattern matching
    Match,

    // =========================================
    // Keywords - Data structures
    // =========================================
    /// `struct` - struct definition
    Struct,
    /// `enum` - enum definition
    Enum,
    /// `impl` - method implementation
    Impl,
    /// `trait` - trait definition
    Trait,
    /// `type` - type alias
    Type,

    // =========================================
    // Keywords - HTTP (language core)
    // =========================================
    /// `api` - HTTP endpoint definition
    Api,
    /// `ws` - WebSocket endpoint definition
    Ws,
    /// `server` - server configuration
    Server,
    /// `middleware` - middleware definition
    Middleware,
    /// `use` - apply middleware
    Use,
    /// `body` - HTTP request body
    Body,
    /// `query` - query parameters
    Query,
    /// `header` - HTTP headers
    Header,

    // =========================================
    // Keywords - HTTP methods
    // =========================================
    /// `GET`
    Get,
    /// `POST`
    Post,
    /// `PUT`
    Put,
    /// `DELETE`
    Delete,
    /// `PATCH`
    Patch,

    // =========================================
    // Keywords - Async
    // =========================================
    /// `async` - asynchronous function
    Async,
    /// `await` - await future
    Await,

    // =========================================
    // Keywords - Database
    // =========================================
    /// `db` - database connection declaration
    Db,
    /// `transaction` - transaction block
    Transaction,

    // =========================================
    // Keywords - Modules
    // =========================================
    /// `module` - module declaration
    Module,
    /// `import` - import module
    Import,
    /// `from` - import from specific module
    From,
    /// `as` - rename import
    As,
    /// `pub` - public visibility
    Pub,

    // =========================================
    // Keywords - Special types
    // =========================================
    /// `None` - null value
    None,
    /// `Some` - present value
    Some,
    /// `Ok` - success result
    Ok,
    /// `Err` - error result
    Err,
    /// `true`
    True,
    /// `false`
    False,
    /// `is` - type comparison
    Is,
    /// `copy` - struct modifier
    Copy,

    // =========================================
    // Primitive types (keywords)
    // =========================================
    /// `int`
    IntType,
    /// `float`
    FloatType,
    /// `bool`
    BoolType,
    /// `string`
    StringType,
    /// `Result`
    ResultType,
    /// `Option`
    OptionType,

    // =========================================
    // Literals
    // =========================================
    /// Integer literal: `42`, `0xFF`, `0b1010`
    IntLit(i64),
    /// Float literal: `3.14`, `2.5e10`
    FloatLit(f64),
    /// String literal: `"hello"`
    StringLit(String),
    /// Interpolated string: f"hello {name}"
    /// Contains pairs of (literal_part, expr_string)
    /// e.g., f"Hello, {name}!" -> [("Hello, ", "name"), ("!", "")]
    InterpolatedString(Vec<(String, String)>),
    /// Identifier: `foo`, `userName`, `_private`
    Ident(String),

    // =========================================
    // Arithmetic operators
    // =========================================
    /// `+`
    Plus,
    /// `-`
    Minus,
    /// `*`
    Star,
    /// `/`
    Slash,
    /// `%`
    Percent,

    // =========================================
    // Comparison operators
    // =========================================
    /// `==`
    EqEq,
    /// `!=`
    Ne,
    /// `<`
    Lt,
    /// `<=`
    Le,
    /// `>`
    Gt,
    /// `>=`
    Ge,

    // =========================================
    // Logical operators
    // =========================================
    /// `and`
    And,
    /// `or`
    Or,
    /// `not`
    Not,

    // =========================================
    // Assignment operators
    // =========================================
    /// `=`
    Eq,
    /// `+=`
    PlusEq,
    /// `-=`
    MinusEq,
    /// `*=`
    StarEq,
    /// `/=`
    SlashEq,

    // =========================================
    // Reference operators (ownership)
    // =========================================
    /// `&`
    Ampersand,
    /// `&mut` (treated as a single token)
    AmpersandMut,

    // =========================================
    // Punctuation and delimiters
    // =========================================
    /// `->`
    Arrow,
    /// `:`
    Colon,
    /// `::`
    ColonColon,
    /// `,`
    Comma,
    /// `.`
    Dot,
    /// `..`
    DotDot,
    /// `..=`
    DotDotEq,
    /// `(`
    LParen,
    /// `)`
    RParen,
    /// `{`
    LBrace,
    /// `}`
    RBrace,
    /// `[`
    LBracket,
    /// `]`
    RBracket,
    /// `<`
    LAngle,
    /// `>`
    RAngle,
    /// `|`
    Pipe,
    /// `$`
    Dollar,
    /// `?`
    Question,

    // =========================================
    // Special tokens (indentation)
    // =========================================
    /// Newline (significant in Mendes)
    Newline,
    /// Indentation increase
    Indent,
    /// Indentation decrease
    Dedent,

    // =========================================
    // End of file
    // =========================================
    /// End of file
    Eof,

    // =========================================
    // Error token
    // =========================================
    /// Invalid character or lexing error
    Error(String),
}

impl TokenKind {
    /// Returns true if the token is a keyword
    pub fn is_keyword(&self) -> bool {
        matches!(
            self,
            TokenKind::Let
                | TokenKind::Mut
                | TokenKind::Fn
                | TokenKind::Return
                | TokenKind::If
                | TokenKind::Else
                | TokenKind::For
                | TokenKind::In
                | TokenKind::While
                | TokenKind::Break
                | TokenKind::Continue
                | TokenKind::Match
                | TokenKind::Struct
                | TokenKind::Enum
                | TokenKind::Impl
                | TokenKind::Trait
                | TokenKind::Type
                | TokenKind::Api
                | TokenKind::Ws
                | TokenKind::Server
                | TokenKind::Middleware
                | TokenKind::Use
                | TokenKind::Body
                | TokenKind::Query
                | TokenKind::Header
                | TokenKind::Get
                | TokenKind::Post
                | TokenKind::Put
                | TokenKind::Delete
                | TokenKind::Patch
                | TokenKind::Async
                | TokenKind::Await
                | TokenKind::Db
                | TokenKind::Transaction
                | TokenKind::Module
                | TokenKind::Import
                | TokenKind::From
                | TokenKind::Pub
                | TokenKind::None
                | TokenKind::Some
                | TokenKind::Ok
                | TokenKind::Err
                | TokenKind::True
                | TokenKind::False
                | TokenKind::Is
                | TokenKind::As
                | TokenKind::Copy
                | TokenKind::And
                | TokenKind::Or
                | TokenKind::Not
        )
    }

    /// Returns true if the token is a literal
    pub fn is_literal(&self) -> bool {
        matches!(
            self,
            TokenKind::IntLit(_) | TokenKind::FloatLit(_) | TokenKind::StringLit(_) | TokenKind::InterpolatedString(_)
        )
    }

    /// Converts a string to a keyword, if it is one
    pub fn keyword_from_str(s: &str) -> Option<TokenKind> {
        match s {
            // Control flow
            "let" => Some(TokenKind::Let),
            "mut" => Some(TokenKind::Mut),
            "fn" => Some(TokenKind::Fn),
            "return" => Some(TokenKind::Return),
            "if" => Some(TokenKind::If),
            "else" => Some(TokenKind::Else),
            "for" => Some(TokenKind::For),
            "in" => Some(TokenKind::In),
            "while" => Some(TokenKind::While),
            "break" => Some(TokenKind::Break),
            "continue" => Some(TokenKind::Continue),
            "match" => Some(TokenKind::Match),

            // Structures
            "struct" => Some(TokenKind::Struct),
            "enum" => Some(TokenKind::Enum),
            "impl" => Some(TokenKind::Impl),
            "trait" => Some(TokenKind::Trait),
            "type" => Some(TokenKind::Type),

            // HTTP
            "api" => Some(TokenKind::Api),
            "ws" => Some(TokenKind::Ws),
            "server" => Some(TokenKind::Server),
            "middleware" => Some(TokenKind::Middleware),
            "use" => Some(TokenKind::Use),
            "body" => Some(TokenKind::Body),
            "query" => Some(TokenKind::Query),
            "header" => Some(TokenKind::Header),

            // HTTP methods
            "GET" => Some(TokenKind::Get),
            "POST" => Some(TokenKind::Post),
            "PUT" => Some(TokenKind::Put),
            "DELETE" => Some(TokenKind::Delete),
            "PATCH" => Some(TokenKind::Patch),

            // Async
            "async" => Some(TokenKind::Async),
            "await" => Some(TokenKind::Await),

            // Database
            "db" => Some(TokenKind::Db),
            "transaction" => Some(TokenKind::Transaction),

            // Modules
            "module" => Some(TokenKind::Module),
            "import" => Some(TokenKind::Import),
            "from" => Some(TokenKind::From),
            "pub" => Some(TokenKind::Pub),

            // Special types
            "None" => Some(TokenKind::None),
            "Some" => Some(TokenKind::Some),
            "Ok" => Some(TokenKind::Ok),
            "Err" => Some(TokenKind::Err),
            "true" => Some(TokenKind::True),
            "false" => Some(TokenKind::False),
            "is" => Some(TokenKind::Is),
            "as" => Some(TokenKind::As),
            "copy" => Some(TokenKind::Copy),

            // Primitive types
            "int" => Some(TokenKind::IntType),
            "float" => Some(TokenKind::FloatType),
            "bool" => Some(TokenKind::BoolType),
            "string" => Some(TokenKind::StringType),
            "Result" => Some(TokenKind::ResultType),
            "Option" => Some(TokenKind::OptionType),

            // Logical operators
            "and" => Some(TokenKind::And),
            "or" => Some(TokenKind::Or),
            "not" => Some(TokenKind::Not),

            _ => None,
        }
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Keywords
            TokenKind::Let => write!(f, "let"),
            TokenKind::Mut => write!(f, "mut"),
            TokenKind::Fn => write!(f, "fn"),
            TokenKind::Return => write!(f, "return"),
            TokenKind::If => write!(f, "if"),
            TokenKind::Else => write!(f, "else"),
            TokenKind::For => write!(f, "for"),
            TokenKind::In => write!(f, "in"),
            TokenKind::While => write!(f, "while"),
            TokenKind::Break => write!(f, "break"),
            TokenKind::Continue => write!(f, "continue"),
            TokenKind::Match => write!(f, "match"),
            TokenKind::Struct => write!(f, "struct"),
            TokenKind::Enum => write!(f, "enum"),
            TokenKind::Impl => write!(f, "impl"),
            TokenKind::Trait => write!(f, "trait"),
            TokenKind::Type => write!(f, "type"),
            TokenKind::Api => write!(f, "api"),
            TokenKind::Ws => write!(f, "ws"),
            TokenKind::Server => write!(f, "server"),
            TokenKind::Middleware => write!(f, "middleware"),
            TokenKind::Use => write!(f, "use"),
            TokenKind::Body => write!(f, "body"),
            TokenKind::Query => write!(f, "query"),
            TokenKind::Header => write!(f, "header"),
            TokenKind::Get => write!(f, "GET"),
            TokenKind::Post => write!(f, "POST"),
            TokenKind::Put => write!(f, "PUT"),
            TokenKind::Delete => write!(f, "DELETE"),
            TokenKind::Patch => write!(f, "PATCH"),
            TokenKind::Async => write!(f, "async"),
            TokenKind::Await => write!(f, "await"),
            TokenKind::Db => write!(f, "db"),
            TokenKind::Transaction => write!(f, "transaction"),
            TokenKind::Module => write!(f, "module"),
            TokenKind::Import => write!(f, "import"),
            TokenKind::From => write!(f, "from"),
            TokenKind::Pub => write!(f, "pub"),
            TokenKind::None => write!(f, "None"),
            TokenKind::Some => write!(f, "Some"),
            TokenKind::Ok => write!(f, "Ok"),
            TokenKind::Err => write!(f, "Err"),
            TokenKind::True => write!(f, "true"),
            TokenKind::False => write!(f, "false"),
            TokenKind::Is => write!(f, "is"),
            TokenKind::As => write!(f, "as"),
            TokenKind::Copy => write!(f, "copy"),
            TokenKind::IntType => write!(f, "int"),
            TokenKind::FloatType => write!(f, "float"),
            TokenKind::BoolType => write!(f, "bool"),
            TokenKind::StringType => write!(f, "string"),
            TokenKind::ResultType => write!(f, "Result"),
            TokenKind::OptionType => write!(f, "Option"),

            // Literals
            TokenKind::IntLit(n) => write!(f, "{}", n),
            TokenKind::FloatLit(n) => write!(f, "{}", n),
            TokenKind::StringLit(s) => write!(f, "\"{}\"", s),
            TokenKind::InterpolatedString(parts) => {
                write!(f, "f\"")?;
                for (lit, expr) in parts {
                    write!(f, "{}", lit)?;
                    if !expr.is_empty() {
                        write!(f, "{{{}}}", expr)?;
                    }
                }
                write!(f, "\"")
            }
            TokenKind::Ident(s) => write!(f, "{}", s),

            // Operators
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Star => write!(f, "*"),
            TokenKind::Slash => write!(f, "/"),
            TokenKind::Percent => write!(f, "%"),
            TokenKind::EqEq => write!(f, "=="),
            TokenKind::Ne => write!(f, "!="),
            TokenKind::Lt => write!(f, "<"),
            TokenKind::Le => write!(f, "<="),
            TokenKind::Gt => write!(f, ">"),
            TokenKind::Ge => write!(f, ">="),
            TokenKind::And => write!(f, "and"),
            TokenKind::Or => write!(f, "or"),
            TokenKind::Not => write!(f, "not"),
            TokenKind::Eq => write!(f, "="),
            TokenKind::PlusEq => write!(f, "+="),
            TokenKind::MinusEq => write!(f, "-="),
            TokenKind::StarEq => write!(f, "*="),
            TokenKind::SlashEq => write!(f, "/="),
            TokenKind::Ampersand => write!(f, "&"),
            TokenKind::AmpersandMut => write!(f, "&mut"),

            // Punctuation
            TokenKind::Arrow => write!(f, "->"),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::ColonColon => write!(f, "::"),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Dot => write!(f, "."),
            TokenKind::DotDot => write!(f, ".."),
            TokenKind::DotDotEq => write!(f, "..="),
            TokenKind::LParen => write!(f, "("),
            TokenKind::RParen => write!(f, ")"),
            TokenKind::LBrace => write!(f, "{{"),
            TokenKind::RBrace => write!(f, "}}"),
            TokenKind::LBracket => write!(f, "["),
            TokenKind::RBracket => write!(f, "]"),
            TokenKind::LAngle => write!(f, "<"),
            TokenKind::RAngle => write!(f, ">"),
            TokenKind::Pipe => write!(f, "|"),
            TokenKind::Dollar => write!(f, "$"),
            TokenKind::Question => write!(f, "?"),

            // Special
            TokenKind::Newline => write!(f, "NEWLINE"),
            TokenKind::Indent => write!(f, "INDENT"),
            TokenKind::Dedent => write!(f, "DEDENT"),
            TokenKind::Eof => write!(f, "EOF"),
            TokenKind::Error(msg) => write!(f, "ERROR({})", msg),
        }
    }
}

/// A token with its location in the source code
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    /// Token type
    pub kind: TokenKind,
    /// Location in source code
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }

    /// Checks if the token is of a specific type
    pub fn is(&self, kind: &TokenKind) -> bool {
        std::mem::discriminant(&self.kind) == std::mem::discriminant(kind)
    }

    /// Checks if it is end of file
    pub fn is_eof(&self) -> bool {
        matches!(self.kind, TokenKind::Eof)
    }

    /// Checks if it is an error
    pub fn is_error(&self) -> bool {
        matches!(self.kind, TokenKind::Error(_))
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} at {}:{}",
            self.kind, self.span.start.line, self.span.start.column
        )
    }
}
