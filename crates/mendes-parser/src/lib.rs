//! mendes-parser - Parser for the Mendes language
//!
//! Converts a sequence of tokens into an AST (Abstract Syntax Tree).
//!
//! # Example
//!
//! ```rust
//! use mendes_lexer::Lexer;
//! use mendes_parser::{Parser, parse};
//!
//! let source = "let x: int = 10\n";
//! let mut lexer = Lexer::new(source, 0);
//! let tokens = lexer.tokenize();
//!
//! let (program, diagnostics) = parse(tokens);
//! println!("Statements: {}", program.statements.len());
//! ```

pub mod ast;
pub mod parser;

pub use ast::*;
pub use parser::{parse, Parser};
