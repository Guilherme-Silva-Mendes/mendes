//! mendes-lexer - Lexer/Tokenizer for the Mendes language
//!
//! This crate converts Mendes source code into a sequence of tokens.
//!
//! # Features
//!
//! - Significant indentation (like Python)
//! - Support for HTTP keywords (api, GET, POST, etc.)
//! - Support for async/await
//! - Literals: integers, floats, strings
//! - Ownership operators (&, &mut)
//!
//! # Example
//!
//! ```rust
//! use mendes_lexer::{Lexer, TokenKind};
//!
//! let source = r#"
//! let x: int = 10
//! let y: string = "hello"
//! "#;
//!
//! let mut lexer = Lexer::new(source, 0);
//! let tokens = lexer.tokenize();
//!
//! for token in &tokens {
//!     println!("{:?}", token.kind);
//! }
//! ```

pub mod lexer;
pub mod token;

pub use lexer::{tokenize, Lexer};
pub use token::{Token, TokenKind};
