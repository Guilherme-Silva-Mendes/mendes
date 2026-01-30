//! mendes-semantic - Semantic analysis for the Mendes language
//!
//! Responsible for:
//! - Type checking
//! - Ownership checking (ownership/move/borrow verification)
//! - Symbol resolution
//! - Async rules validation
//!
//! # Example
//!
//! ```rust
//! use mendes_parser::{parse, Program};
//! use mendes_lexer::Lexer;
//! use mendes_semantic::{analyze, SemanticContext};
//!
//! let source = "let x: int = 10\n";
//! let mut lexer = Lexer::new(source, 0);
//! let tokens = lexer.tokenize();
//! let (program, _) = parse(tokens);
//!
//! let mut ctx = SemanticContext::new();
//! let diagnostics = analyze(&program, &mut ctx);
//! ```

pub mod types;
pub mod symbols;
pub mod checker;
pub mod ownership;

pub use checker::{analyze, TypeChecker};
pub use symbols::{Symbol, SymbolTable, SymbolKind};
pub use types::{MendesType, TypeId};

/// Semantic analysis context
#[derive(Debug, Default)]
pub struct SemanticContext {
    /// Global symbol table
    pub symbols: SymbolTable,
    /// Registered types
    pub types: types::TypeRegistry,
}

impl SemanticContext {
    pub fn new() -> Self {
        let mut ctx = Self::default();
        ctx.register_builtins();
        ctx
    }

    /// Registers built-in types and functions
    fn register_builtins(&mut self) {
        // Primitive types are already in the TypeRegistry by default

        // `db` is a global namespace for database connections
        // Access: db.{connection_name}.query(...), db.{connection_name}.execute(...)
        self.symbols.define(Symbol {
            name: "db".to_string(),
            kind: SymbolKind::Variable,
            ty: MendesType::Named("DatabaseNamespace".to_string()),
            mutable: false,
            defined_at: None,
        });

        // Built-in functions

        // print(value: any) -> ()
        self.symbols.define(Symbol {
            name: "print".to_string(),
            kind: SymbolKind::Function {
                generic_params: vec![],
                params: vec![("value".to_string(), MendesType::Any)],
                return_type: MendesType::Unit,
                is_async: false,
            },
            ty: MendesType::Function {
                params: vec![MendesType::Any],
                ret: Box::new(MendesType::Unit),
            },
            mutable: false,
            defined_at: None,
        });

        // println(value: any) -> ()
        self.symbols.define(Symbol {
            name: "println".to_string(),
            kind: SymbolKind::Function {
                generic_params: vec![],
                params: vec![("value".to_string(), MendesType::Any)],
                return_type: MendesType::Unit,
                is_async: false,
            },
            ty: MendesType::Function {
                params: vec![MendesType::Any],
                ret: Box::new(MendesType::Unit),
            },
            mutable: false,
            defined_at: None,
        });

        // len(collection: any) -> int
        self.symbols.define(Symbol {
            name: "len".to_string(),
            kind: SymbolKind::Function {
                generic_params: vec![],
                params: vec![("collection".to_string(), MendesType::Any)],
                return_type: MendesType::Int,
                is_async: false,
            },
            ty: MendesType::Function {
                params: vec![MendesType::Any],
                ret: Box::new(MendesType::Int),
            },
            mutable: false,
            defined_at: None,
        });

        // str(value: any) -> string
        self.symbols.define(Symbol {
            name: "str".to_string(),
            kind: SymbolKind::Function {
                generic_params: vec![],
                params: vec![("value".to_string(), MendesType::Any)],
                return_type: MendesType::String,
                is_async: false,
            },
            ty: MendesType::Function {
                params: vec![MendesType::Any],
                ret: Box::new(MendesType::String),
            },
            mutable: false,
            defined_at: None,
        });

        // int(value: string) -> int
        self.symbols.define(Symbol {
            name: "int".to_string(),
            kind: SymbolKind::Function {
                generic_params: vec![],
                params: vec![("value".to_string(), MendesType::String)],
                return_type: MendesType::Int,
                is_async: false,
            },
            ty: MendesType::Function {
                params: vec![MendesType::String],
                ret: Box::new(MendesType::Int),
            },
            mutable: false,
            defined_at: None,
        });

        // float(value: string) -> float
        self.symbols.define(Symbol {
            name: "float".to_string(),
            kind: SymbolKind::Function {
                generic_params: vec![],
                params: vec![("value".to_string(), MendesType::String)],
                return_type: MendesType::Float,
                is_async: false,
            },
            ty: MendesType::Function {
                params: vec![MendesType::String],
                ret: Box::new(MendesType::Float),
            },
            mutable: false,
            defined_at: None,
        });

        // log(msg: string) -> ()
        self.symbols.define(Symbol {
            name: "log".to_string(),
            kind: SymbolKind::Function {
                generic_params: vec![],
                params: vec![("msg".to_string(), MendesType::String)],
                return_type: MendesType::Unit,
                is_async: false,
            },
            ty: MendesType::Function {
                params: vec![MendesType::String],
                ret: Box::new(MendesType::Unit),
            },
            mutable: false,
            defined_at: None,
        });

        // HttpError(status: int, message: string) -> HttpError
        self.symbols.define(Symbol {
            name: "HttpError".to_string(),
            kind: SymbolKind::Function {
                generic_params: vec![],
                params: vec![
                    ("status".to_string(), MendesType::Int),
                    ("message".to_string(), MendesType::String),
                ],
                return_type: MendesType::Named("HttpError".to_string()),
                is_async: false,
            },
            ty: MendesType::Function {
                params: vec![MendesType::Int, MendesType::String],
                ret: Box::new(MendesType::Named("HttpError".to_string())),
            },
            mutable: false,
            defined_at: None,
        });
    }
}
