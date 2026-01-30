//! mendes-error - Diagnostics system for the Mendes language
//!
//! This crate provides structures for reporting compilation errors
//! in a clear and detailed way, similar to the Rust compiler style.
//!
//! # Example
//!
//! ```rust
//! use mendes_error::{Diagnostic, ErrorCode, SourceCache, DiagnosticRenderer};
//! use mendes_error::span::{Span, Position};
//!
//! let mut cache = SourceCache::new();
//! let file_id = cache.add("example.ms", "let x = 10");
//!
//! let span = Span::new(
//!     Position::new(1, 5, 4),
//!     Position::new(1, 6, 5),
//!     file_id,
//! );
//!
//! let diagnostic = Diagnostic::error("undeclared variable")
//!     .with_code(ErrorCode::UNKNOWN_VARIABLE)
//!     .with_label(span, "not found in this scope");
//!
//! let renderer = DiagnosticRenderer::new(&cache);
//! println!("{}", renderer.render(&diagnostic));
//! ```

pub mod diagnostic;
pub mod span;

pub use diagnostic::{
    Diagnostic, DiagnosticRenderer, ErrorCode, Label, Level, SourceCache, SourceFile, Suggestion,
};
pub use span::{Position, Span, Spanned};

/// Default Result type for operations that may fail with diagnostics
pub type Result<T> = std::result::Result<T, Diagnostic>;

/// Collection of diagnostics accumulated during compilation
#[derive(Debug, Default)]
pub struct Diagnostics {
    items: Vec<Diagnostic>,
}

impl Diagnostics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, diagnostic: Diagnostic) {
        self.items.push(diagnostic);
    }

    pub fn error(&mut self, message: impl Into<String>) {
        self.items.push(Diagnostic::error(message));
    }

    pub fn warning(&mut self, message: impl Into<String>) {
        self.items.push(Diagnostic::warning(message));
    }

    pub fn has_errors(&self) -> bool {
        self.items.iter().any(|d| d.level == Level::Error)
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Diagnostic> {
        self.items.iter()
    }

    /// Renders all diagnostics
    pub fn render(&self, cache: &SourceCache) -> String {
        let renderer = DiagnosticRenderer::new(cache);
        self.items
            .iter()
            .map(|d| renderer.render(d))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl IntoIterator for Diagnostics {
    type Item = Diagnostic;
    type IntoIter = std::vec::IntoIter<Diagnostic>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}
