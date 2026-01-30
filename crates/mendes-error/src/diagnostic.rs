//! Diagnostic - Rust-style error message system
//!
//! Generates detailed error messages with:
//! - Error code (E0001, E0002, etc.)
//! - Precise location
//! - Source code snippet
//! - Fix suggestions

use crate::span::Span;
use std::fmt;

/// Diagnostic severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    /// Fatal error - prevents compilation
    Error,
    /// Warning - does not prevent compilation
    Warning,
    /// Note - additional information
    Note,
    /// Help - fix suggestion
    Help,
}

impl Level {
    pub fn as_str(&self) -> &'static str {
        match self {
            Level::Error => "error",
            Level::Warning => "warning",
            Level::Note => "note",
            Level::Help => "help",
        }
    }

    /// Returns the ANSI code for coloring (if terminal supports it)
    pub fn color_code(&self) -> &'static str {
        match self {
            Level::Error => "\x1b[1;31m",   // Bold Red
            Level::Warning => "\x1b[1;33m", // Bold Yellow
            Level::Note => "\x1b[1;36m",    // Bold Cyan
            Level::Help => "\x1b[1;32m",    // Bold Green
        }
    }
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A fix suggestion
#[derive(Debug, Clone)]
pub struct Suggestion {
    /// Suggestion message
    pub message: String,
    /// Span where to apply the suggestion (optional)
    pub span: Option<Span>,
    /// Suggested replacement text (optional)
    pub replacement: Option<String>,
}

impl Suggestion {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            span: None,
            replacement: None,
        }
    }

    pub fn with_replacement(mut self, span: Span, replacement: impl Into<String>) -> Self {
        self.span = Some(span);
        self.replacement = Some(replacement.into());
        self
    }
}

/// A label pointing to a specific region of the code
#[derive(Debug, Clone)]
pub struct Label {
    /// Span of the region
    pub span: Span,
    /// Label message
    pub message: String,
    /// Whether this is the primary or secondary label
    pub primary: bool,
}

impl Label {
    pub fn primary(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            message: message.into(),
            primary: true,
        }
    }

    pub fn secondary(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            message: message.into(),
            primary: false,
        }
    }
}

/// Structured error code
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ErrorCode {
    /// Category (L = Lexer, P = Parser, T = Type, O = Ownership)
    pub category: char,
    /// Error number
    pub number: u16,
}

impl ErrorCode {
    pub const fn new(category: char, number: u16) -> Self {
        Self { category, number }
    }

    // Lexer errors
    pub const UNEXPECTED_CHAR: Self = Self::new('L', 1);
    pub const UNTERMINATED_STRING: Self = Self::new('L', 2);
    pub const INVALID_NUMBER: Self = Self::new('L', 3);
    pub const INVALID_INDENT: Self = Self::new('L', 4);

    // Parser errors
    pub const UNEXPECTED_TOKEN: Self = Self::new('P', 1);
    pub const EXPECTED_EXPRESSION: Self = Self::new('P', 2);
    pub const EXPECTED_TYPE: Self = Self::new('P', 3);
    pub const INVALID_SYNTAX: Self = Self::new('P', 4);

    // Type errors
    pub const TYPE_MISMATCH: Self = Self::new('T', 1);
    pub const UNKNOWN_TYPE: Self = Self::new('T', 2);
    pub const UNKNOWN_VARIABLE: Self = Self::new('T', 3);

    // Ownership errors
    pub const USE_AFTER_MOVE: Self = Self::new('O', 1);
    pub const BORROW_AFTER_MOVE: Self = Self::new('O', 2);
    pub const MUT_BORROW_CONFLICT: Self = Self::new('O', 3);
    pub const BORROW_ACROSS_AWAIT: Self = Self::new('O', 4);
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "E{}{:03}", self.category, self.number)
    }
}

/// A complete diagnostic
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// Severity level
    pub level: Level,
    /// Error code (optional)
    pub code: Option<ErrorCode>,
    /// Main message
    pub message: String,
    /// Labels pointing to the code
    pub labels: Vec<Label>,
    /// Additional notes
    pub notes: Vec<String>,
    /// Fix suggestions
    pub suggestions: Vec<Suggestion>,
}

impl Diagnostic {
    /// Creates a new error
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            level: Level::Error,
            code: None,
            message: message.into(),
            labels: Vec::new(),
            notes: Vec::new(),
            suggestions: Vec::new(),
        }
    }

    /// Creates a new warning
    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            level: Level::Warning,
            code: None,
            message: message.into(),
            labels: Vec::new(),
            notes: Vec::new(),
            suggestions: Vec::new(),
        }
    }

    /// Sets the error code
    pub fn with_code(mut self, code: ErrorCode) -> Self {
        self.code = Some(code);
        self
    }

    /// Adds a primary label
    pub fn with_label(mut self, span: Span, message: impl Into<String>) -> Self {
        self.labels.push(Label::primary(span, message));
        self
    }

    /// Adds a secondary label
    pub fn with_secondary_label(mut self, span: Span, message: impl Into<String>) -> Self {
        self.labels.push(Label::secondary(span, message));
        self
    }

    /// Adds a note
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    /// Adds a suggestion
    pub fn with_suggestion(mut self, suggestion: Suggestion) -> Self {
        self.suggestions.push(suggestion);
        self
    }

    /// Adds a simple suggestion (text only)
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.suggestions.push(Suggestion::new(help));
        self
    }
}

/// Stores information about source files for rendering diagnostics
#[derive(Debug, Default)]
pub struct SourceCache {
    files: Vec<SourceFile>,
}

#[derive(Debug)]
pub struct SourceFile {
    pub name: String,
    pub source: String,
    /// Offset of each line (for fast lookup)
    line_starts: Vec<usize>,
}

impl SourceFile {
    pub fn new(name: impl Into<String>, source: impl Into<String>) -> Self {
        let source = source.into();
        let line_starts = std::iter::once(0)
            .chain(source.match_indices('\n').map(|(i, _)| i + 1))
            .collect();

        Self {
            name: name.into(),
            source,
            line_starts,
        }
    }

    /// Returns the line of code (0-indexed internally, but line is 1-indexed)
    pub fn get_line(&self, line: u32) -> Option<&str> {
        let line_idx = line.checked_sub(1)? as usize;
        let start = *self.line_starts.get(line_idx)?;
        let end = self
            .line_starts
            .get(line_idx + 1)
            .map(|&e| e.saturating_sub(1))
            .unwrap_or(self.source.len());

        Some(&self.source[start..end])
    }
}

impl SourceCache {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a file and returns its ID
    pub fn add(&mut self, name: impl Into<String>, source: impl Into<String>) -> u32 {
        let id = self.files.len() as u32;
        self.files.push(SourceFile::new(name, source));
        id
    }

    pub fn get(&self, id: u32) -> Option<&SourceFile> {
        self.files.get(id as usize)
    }
}

/// Renders a diagnostic for display
pub struct DiagnosticRenderer<'a> {
    cache: &'a SourceCache,
    use_colors: bool,
}

impl<'a> DiagnosticRenderer<'a> {
    pub fn new(cache: &'a SourceCache) -> Self {
        Self {
            cache,
            use_colors: true,
        }
    }

    pub fn without_colors(mut self) -> Self {
        self.use_colors = false;
        self
    }

    /// Renders the diagnostic as a string
    pub fn render(&self, diagnostic: &Diagnostic) -> String {
        let mut output = String::new();

        // Line 1: error[E0001]: message
        let reset = if self.use_colors { "\x1b[0m" } else { "" };
        let color = if self.use_colors {
            diagnostic.level.color_code()
        } else {
            ""
        };
        let bold = if self.use_colors { "\x1b[1m" } else { "" };

        output.push_str(color);
        output.push_str(diagnostic.level.as_str());

        if let Some(code) = &diagnostic.code {
            output.push('[');
            output.push_str(&code.to_string());
            output.push(']');
        }

        output.push_str(reset);
        output.push_str(bold);
        output.push_str(": ");
        output.push_str(&diagnostic.message);
        output.push_str(reset);
        output.push('\n');

        // Labels with code snippets
        for label in &diagnostic.labels {
            if let Some(file) = self.cache.get(label.span.file_id) {
                // --> file:line:column
                let blue = if self.use_colors { "\x1b[1;34m" } else { "" };

                output.push_str(&format!(
                    " {}-->{} {}:{}:{}\n",
                    blue,
                    reset,
                    file.name,
                    label.span.start.line,
                    label.span.start.column
                ));

                // Line number and code
                if let Some(line_content) = file.get_line(label.span.start.line) {
                    let line_num = label.span.start.line;
                    let line_num_width = line_num.to_string().len();
                    let padding = " ".repeat(line_num_width);

                    // Empty line with bar
                    output.push_str(&format!(" {} {}|{}\n", padding, blue, reset));

                    // Line with code
                    output.push_str(&format!(
                        " {}{}{} |{} {}\n",
                        blue, line_num, reset, reset, line_content
                    ));

                    // Line with underline
                    let col_start = label.span.start.column as usize;
                    let underline_len = if label.span.start.line == label.span.end.line {
                        (label.span.end.column - label.span.start.column).max(1) as usize
                    } else {
                        line_content.len().saturating_sub(col_start - 1).max(1)
                    };

                    let spaces = " ".repeat(col_start.saturating_sub(1));
                    let underline_char = if label.primary { '^' } else { '-' };
                    let underline = underline_char.to_string().repeat(underline_len);

                    let label_color = if label.primary { color } else { blue };

                    output.push_str(&format!(
                        " {} {}|{} {}{}{} {}\n",
                        padding, blue, reset, spaces, label_color, underline, label.message
                    ));
                    output.push_str(reset);
                }
            }
        }

        // Notes
        for note in &diagnostic.notes {
            output.push_str(&format!(" {} = {}{}: {}\n", " ", bold, "note", note));
            output.push_str(reset);
        }

        // Suggestions
        for suggestion in &diagnostic.suggestions {
            let green = if self.use_colors { "\x1b[1;32m" } else { "" };
            output.push_str(&format!(
                " {} = {}help{}: {}\n",
                " ", green, reset, suggestion.message
            ));
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::span::Position;

    #[test]
    fn test_diagnostic_rendering() {
        let mut cache = SourceCache::new();
        let file_id = cache.add("test.ms", "let x: int = 10\nlet y = x + 1");

        let span = Span::new(
            Position::new(1, 5, 4),
            Position::new(1, 6, 5),
            file_id,
        );

        let diagnostic = Diagnostic::error("invalid type")
            .with_code(ErrorCode::TYPE_MISMATCH)
            .with_label(span, "expected `int`, found `string`")
            .with_help("consider converting the value to int");

        let renderer = DiagnosticRenderer::new(&cache).without_colors();
        let output = renderer.render(&diagnostic);

        assert!(output.contains("error[ET001]"));
        assert!(output.contains("invalid type"));
        assert!(output.contains("test.ms:1:5"));
    }
}
