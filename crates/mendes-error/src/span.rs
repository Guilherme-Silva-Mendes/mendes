//! Span - Source code location
//!
//! A Span represents a region in the source code, used to
//! report errors with precision.

/// Represents a position in the source code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Position {
    /// Line (1-indexed)
    pub line: u32,
    /// Column (1-indexed)
    pub column: u32,
    /// Byte offset from the beginning of the file
    pub offset: usize,
}

impl Position {
    pub fn new(line: u32, column: u32, offset: usize) -> Self {
        Self { line, column, offset }
    }
}

/// Represents a region in the source code (start to end)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    /// Start position
    pub start: Position,
    /// End position
    pub end: Position,
    /// Source file ID (to support multiple files)
    pub file_id: u32,
}

impl Span {
    pub fn new(start: Position, end: Position, file_id: u32) -> Self {
        Self { start, end, file_id }
    }

    /// Creates a span from a single position
    pub fn point(pos: Position, file_id: u32) -> Self {
        Self {
            start: pos,
            end: pos,
            file_id,
        }
    }

    /// Combines two spans, creating one that covers both
    pub fn merge(self, other: Span) -> Span {
        debug_assert_eq!(self.file_id, other.file_id, "Cannot merge spans from different files");
        Span {
            start: if self.start.offset < other.start.offset {
                self.start
            } else {
                other.start
            },
            end: if self.end.offset > other.end.offset {
                self.end
            } else {
                other.end
            },
            file_id: self.file_id,
        }
    }

    /// Returns the length in bytes
    pub fn len(&self) -> usize {
        self.end.offset.saturating_sub(self.start.offset)
    }

    /// Checks if the span is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Trait for types that have a location in the code
pub trait Spanned {
    fn span(&self) -> Span;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_merge() {
        let span1 = Span::new(
            Position::new(1, 1, 0),
            Position::new(1, 5, 4),
            0,
        );
        let span2 = Span::new(
            Position::new(1, 10, 9),
            Position::new(1, 15, 14),
            0,
        );

        let merged = span1.merge(span2);
        assert_eq!(merged.start.offset, 0);
        assert_eq!(merged.end.offset, 14);
    }

    #[test]
    fn test_span_len() {
        let span = Span::new(
            Position::new(1, 1, 0),
            Position::new(1, 10, 9),
            0,
        );
        assert_eq!(span.len(), 9);
    }
}
