//! Lexer for the Mendes language
//!
//! Converts source code into a sequence of tokens.
//! Supports significant indentation (Python style).

use crate::token::{Token, TokenKind};
use mendes_error::{
    span::{Position, Span},
    Diagnostic, Diagnostics, ErrorCode,
};

/// The Mendes language Lexer
pub struct Lexer<'src> {
    /// Source code being analyzed
    source: &'src str,
    /// Source code characters
    chars: Vec<char>,
    /// Current position (index in chars vector)
    pos: usize,
    /// Current line (1-indexed)
    line: u32,
    /// Current column (1-indexed)
    column: u32,
    /// Byte offset
    offset: usize,
    /// Source file ID
    file_id: u32,
    /// Stack of indentation levels
    indent_stack: Vec<u32>,
    /// Pending DEDENT tokens to emit
    pending_dedents: u32,
    /// Whether we are at the start of a line (to process indentation)
    at_line_start: bool,
    /// Accumulated diagnostics
    diagnostics: Diagnostics,
}

impl<'src> Lexer<'src> {
    /// Creates a new lexer for the given source code
    pub fn new(source: &'src str, file_id: u32) -> Self {
        Self {
            source,
            chars: source.chars().collect(),
            pos: 0,
            line: 1,
            column: 1,
            offset: 0,
            file_id,
            indent_stack: vec![0], // Start with indentation 0
            pending_dedents: 0,
            at_line_start: true,
            diagnostics: Diagnostics::new(),
        }
    }

    /// Returns the accumulated diagnostics
    pub fn diagnostics(&self) -> &Diagnostics {
        &self.diagnostics
    }

    /// Consumes and returns the diagnostics
    pub fn take_diagnostics(&mut self) -> Diagnostics {
        std::mem::take(&mut self.diagnostics)
    }

    /// Returns the current character without advancing
    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    /// Returns the next character without advancing
    fn peek_next(&self) -> Option<char> {
        self.chars.get(self.pos + 1).copied()
    }

    /// Advances to the next character
    fn advance(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.pos += 1;
        self.offset += ch.len_utf8();

        if ch == '\n' {
            self.line += 1;
            self.column = 1;
            self.at_line_start = true;
        } else {
            self.column += 1;
        }

        Some(ch)
    }

    /// Creates a position at the current location
    fn current_position(&self) -> Position {
        Position::new(self.line, self.column, self.offset)
    }

    /// Creates a span from a position to the current location
    fn make_span(&self, start: Position) -> Span {
        Span::new(start, self.current_position(), self.file_id)
    }

    /// Skips whitespace (except newlines)
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == ' ' || ch == '\t' || ch == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Skips comments (# until end of line)
    fn skip_comment(&mut self) {
        if self.peek() == Some('#') {
            while let Some(ch) = self.peek() {
                if ch == '\n' {
                    break;
                }
                self.advance();
            }
        }
    }

    /// Counts indentation spaces at the beginning of a line
    fn count_indent(&mut self) -> u32 {
        let mut spaces = 0u32;

        while let Some(ch) = self.peek() {
            match ch {
                ' ' => {
                    spaces += 1;
                    self.advance();
                }
                '\t' => {
                    // Tab = 4 spaces
                    spaces += 4;
                    self.advance();
                }
                _ => break,
            }
        }

        spaces
    }

    /// Processes indentation and returns INDENT/DEDENT tokens if necessary
    fn handle_indentation(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        let start = self.current_position();

        let indent = self.count_indent();
        let current_indent = *self.indent_stack.last().unwrap();

        if indent > current_indent {
            // Indentation increased
            self.indent_stack.push(indent);
            tokens.push(Token::new(TokenKind::Indent, self.make_span(start)));
        } else if indent < current_indent {
            // Indentation decreased - may need multiple DEDENTs
            while let Some(&level) = self.indent_stack.last() {
                if level > indent {
                    self.indent_stack.pop();
                    tokens.push(Token::new(TokenKind::Dedent, self.make_span(start)));
                } else if level < indent {
                    // Inconsistent indentation
                    self.diagnostics.push(
                        Diagnostic::error("inconsistent indentation")
                            .with_code(ErrorCode::INVALID_INDENT)
                            .with_label(self.make_span(start), "indentation does not match any previous level")
                            .with_help("use consistent indentation (4 spaces recommended)")
                    );
                    break;
                } else {
                    break;
                }
            }
        }
        // If indent == current_indent, emit nothing

        self.at_line_start = false;
        tokens
    }

    /// Reads a number (integer or float)
    fn read_number(&mut self) -> Token {
        let start = self.current_position();
        let mut num_str = String::new();
        let mut is_float = false;

        // Check prefix (0x, 0b, 0o)
        if self.peek() == Some('0') {
            num_str.push(self.advance().unwrap());

            match self.peek() {
                Some('x') | Some('X') => {
                    num_str.push(self.advance().unwrap());
                    while let Some(ch) = self.peek() {
                        if ch.is_ascii_hexdigit() || ch == '_' {
                            if ch != '_' {
                                num_str.push(ch);
                            }
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    let value = i64::from_str_radix(&num_str[2..], 16).unwrap_or(0);
                    return Token::new(TokenKind::IntLit(value), self.make_span(start));
                }
                Some('b') | Some('B') => {
                    num_str.push(self.advance().unwrap());
                    while let Some(ch) = self.peek() {
                        if ch == '0' || ch == '1' || ch == '_' {
                            if ch != '_' {
                                num_str.push(ch);
                            }
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    let value = i64::from_str_radix(&num_str[2..], 2).unwrap_or(0);
                    return Token::new(TokenKind::IntLit(value), self.make_span(start));
                }
                Some('o') | Some('O') => {
                    num_str.push(self.advance().unwrap());
                    while let Some(ch) = self.peek() {
                        if ch.is_digit(8) || ch == '_' {
                            if ch != '_' {
                                num_str.push(ch);
                            }
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    let value = i64::from_str_radix(&num_str[2..], 8).unwrap_or(0);
                    return Token::new(TokenKind::IntLit(value), self.make_span(start));
                }
                _ => {}
            }
        }

        // Decimal number
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() || ch == '_' {
                if ch != '_' {
                    num_str.push(ch);
                }
                self.advance();
            } else {
                break;
            }
        }

        // Decimal part
        if self.peek() == Some('.') && self.peek_next().map_or(false, |c| c.is_ascii_digit()) {
            is_float = true;
            num_str.push(self.advance().unwrap());

            while let Some(ch) = self.peek() {
                if ch.is_ascii_digit() || ch == '_' {
                    if ch != '_' {
                        num_str.push(ch);
                    }
                    self.advance();
                } else {
                    break;
                }
            }
        }

        // Exponent
        if let Some('e' | 'E') = self.peek() {
            is_float = true;
            num_str.push(self.advance().unwrap());

            if let Some('+' | '-') = self.peek() {
                num_str.push(self.advance().unwrap());
            }

            while let Some(ch) = self.peek() {
                if ch.is_ascii_digit() {
                    num_str.push(ch);
                    self.advance();
                } else {
                    break;
                }
            }
        }

        let span = self.make_span(start);

        if is_float {
            match num_str.parse::<f64>() {
                Ok(value) => Token::new(TokenKind::FloatLit(value), span),
                Err(_) => {
                    self.diagnostics.push(
                        Diagnostic::error("invalid float number")
                            .with_code(ErrorCode::INVALID_NUMBER)
                            .with_label(span, "could not convert to float"),
                    );
                    Token::new(TokenKind::FloatLit(0.0), span)
                }
            }
        } else {
            match num_str.parse::<i64>() {
                Ok(value) => Token::new(TokenKind::IntLit(value), span),
                Err(_) => {
                    self.diagnostics.push(
                        Diagnostic::error("invalid integer number")
                            .with_code(ErrorCode::INVALID_NUMBER)
                            .with_label(span, "could not convert to integer"),
                    );
                    Token::new(TokenKind::IntLit(0), span)
                }
            }
        }
    }

    /// Reads a string
    fn read_string(&mut self) -> Token {
        let start = self.current_position();
        let quote = self.advance().unwrap(); // Consume the opening quote
        let mut value = String::new();

        loop {
            match self.peek() {
                None | Some('\n') => {
                    let span = self.make_span(start);
                    self.diagnostics.push(
                        Diagnostic::error("unterminated string")
                            .with_code(ErrorCode::UNTERMINATED_STRING)
                            .with_label(span, "string starts here but was not closed")
                            .with_help(format!("add {} at the end of the string", quote)),
                    );
                    return Token::new(TokenKind::StringLit(value), span);
                }
                Some('\\') => {
                    self.advance(); // Consume \
                    match self.peek() {
                        Some('n') => {
                            value.push('\n');
                            self.advance();
                        }
                        Some('t') => {
                            value.push('\t');
                            self.advance();
                        }
                        Some('r') => {
                            value.push('\r');
                            self.advance();
                        }
                        Some('\\') => {
                            value.push('\\');
                            self.advance();
                        }
                        Some('"') => {
                            value.push('"');
                            self.advance();
                        }
                        Some('\'') => {
                            value.push('\'');
                            self.advance();
                        }
                        Some('0') => {
                            value.push('\0');
                            self.advance();
                        }
                        Some(ch) => {
                            // Invalid escape, keep the character
                            value.push(ch);
                            self.advance();
                        }
                        None => break,
                    }
                }
                Some(ch) if ch == quote => {
                    self.advance(); // Consume the closing quote
                    break;
                }
                Some(ch) => {
                    value.push(ch);
                    self.advance();
                }
            }
        }

        Token::new(TokenKind::StringLit(value), self.make_span(start))
    }

    /// Reads an identifier or keyword
    fn read_identifier(&mut self) -> Token {
        let start = self.current_position();
        let mut ident = String::new();

        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                ident.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        // Check for f-string: f"..." or f'...'
        if ident == "f" && matches!(self.peek(), Some('"') | Some('\'')) {
            return self.read_interpolated_string(start);
        }

        let span = self.make_span(start);

        // Check if it's a keyword
        let kind = TokenKind::keyword_from_str(&ident).unwrap_or(TokenKind::Ident(ident));

        Token::new(kind, span)
    }

    /// Reads an interpolated string: f"hello {name}!"
    fn read_interpolated_string(&mut self, start: Position) -> Token {
        let quote = self.advance().unwrap(); // Consume the opening quote
        let mut parts: Vec<(String, String)> = Vec::new();
        let mut current_lit = String::new();

        loop {
            match self.peek() {
                None | Some('\n') => {
                    let span = self.make_span(start);
                    self.diagnostics.push(
                        Diagnostic::error("unterminated interpolated string")
                            .with_code(ErrorCode::UNTERMINATED_STRING)
                            .with_label(span, "string starts here but was not closed")
                            .with_help(format!("add {} at the end of the string", quote)),
                    );
                    // Add remaining literal
                    if !current_lit.is_empty() || parts.is_empty() {
                        parts.push((current_lit, String::new()));
                    }
                    return Token::new(TokenKind::InterpolatedString(parts), span);
                }
                Some('\\') => {
                    self.advance(); // Consume \
                    match self.peek() {
                        Some('n') => { current_lit.push('\n'); self.advance(); }
                        Some('t') => { current_lit.push('\t'); self.advance(); }
                        Some('r') => { current_lit.push('\r'); self.advance(); }
                        Some('\\') => { current_lit.push('\\'); self.advance(); }
                        Some('"') => { current_lit.push('"'); self.advance(); }
                        Some('\'') => { current_lit.push('\''); self.advance(); }
                        Some('{') => { current_lit.push('{'); self.advance(); }
                        Some('}') => { current_lit.push('}'); self.advance(); }
                        Some(ch) => { current_lit.push(ch); self.advance(); }
                        None => break,
                    }
                }
                Some('{') => {
                    self.advance(); // Consume {
                    // Read the expression until }
                    let mut expr = String::new();
                    let mut brace_depth = 1;

                    while let Some(ch) = self.peek() {
                        if ch == '{' {
                            brace_depth += 1;
                            expr.push(ch);
                            self.advance();
                        } else if ch == '}' {
                            brace_depth -= 1;
                            if brace_depth == 0 {
                                self.advance(); // Consume closing }
                                break;
                            }
                            expr.push(ch);
                            self.advance();
                        } else if ch == '\n' {
                            break; // Error: newline in interpolation
                        } else {
                            expr.push(ch);
                            self.advance();
                        }
                    }

                    parts.push((current_lit.clone(), expr));
                    current_lit.clear();
                }
                Some(ch) if ch == quote => {
                    self.advance(); // Consume the closing quote
                    break;
                }
                Some(ch) => {
                    current_lit.push(ch);
                    self.advance();
                }
            }
        }

        // Add any trailing literal
        if !current_lit.is_empty() || parts.is_empty() {
            parts.push((current_lit, String::new()));
        }

        Token::new(TokenKind::InterpolatedString(parts), self.make_span(start))
    }

    /// Reads the next token
    pub fn next_token(&mut self) -> Token {
        // First, check for pending DEDENTs
        if self.pending_dedents > 0 {
            self.pending_dedents -= 1;
            return Token::new(TokenKind::Dedent, Span::point(self.current_position(), self.file_id));
        }

        // Skip comments
        self.skip_comment();

        // If we are at the start of a line, process indentation
        if self.at_line_start {
            // Skip blank lines
            while self.peek() == Some('\n') {
                self.advance();
            }

            if self.peek().is_some() {
                let indent_tokens = self.handle_indentation();
                if let Some(token) = indent_tokens.into_iter().next() {
                    // If there is more than one DEDENT, store the pending ones
                    // (simplification - in practice would need a queue)
                    return token;
                }
            }
        }

        // Skip spaces
        self.skip_whitespace();

        // Skip comments after spaces
        self.skip_comment();

        let start = self.current_position();

        // Check for end of file
        let ch = match self.peek() {
            Some(ch) => ch,
            None => {
                // Emit DEDENTs to close all open blocks
                if self.indent_stack.len() > 1 {
                    self.indent_stack.pop();
                    return Token::new(TokenKind::Dedent, self.make_span(start));
                }
                return Token::new(TokenKind::Eof, self.make_span(start));
            }
        };

        // Newline
        if ch == '\n' {
            self.advance();
            return Token::new(TokenKind::Newline, self.make_span(start));
        }

        // Numbers
        if ch.is_ascii_digit() {
            return self.read_number();
        }

        // Strings
        if ch == '"' || ch == '\'' {
            return self.read_string();
        }

        // Identifiers and keywords
        if ch.is_alphabetic() || ch == '_' {
            return self.read_identifier();
        }

        // Operators and punctuation
        self.advance();
        let kind = match ch {
            '+' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::PlusEq
                } else {
                    TokenKind::Plus
                }
            }
            '-' => {
                if self.peek() == Some('>') {
                    self.advance();
                    TokenKind::Arrow
                } else if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::MinusEq
                } else {
                    TokenKind::Minus
                }
            }
            '*' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::StarEq
                } else {
                    TokenKind::Star
                }
            }
            '/' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::SlashEq
                } else {
                    TokenKind::Slash
                }
            }
            '%' => TokenKind::Percent,
            '=' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::EqEq
                } else {
                    TokenKind::Eq
                }
            }
            '!' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::Ne
                } else {
                    TokenKind::Not
                }
            }
            '<' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::Le
                } else {
                    TokenKind::Lt
                }
            }
            '>' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::Ge
                } else {
                    TokenKind::Gt
                }
            }
            '&' => {
                // Check if it's &mut
                if self.peek() == Some('m') {
                    let saved_pos = self.pos;
                    let saved_offset = self.offset;
                    let saved_column = self.column;

                    if self.advance() == Some('m')
                        && self.advance() == Some('u')
                        && self.advance() == Some('t')
                        && !self.peek().map_or(false, |c| c.is_alphanumeric() || c == '_')
                    {
                        TokenKind::AmpersandMut
                    } else {
                        // Backtrack
                        self.pos = saved_pos;
                        self.offset = saved_offset;
                        self.column = saved_column;
                        TokenKind::Ampersand
                    }
                } else {
                    TokenKind::Ampersand
                }
            }
            ':' => {
                if self.peek() == Some(':') {
                    self.advance();
                    TokenKind::ColonColon
                } else {
                    TokenKind::Colon
                }
            }
            ',' => TokenKind::Comma,
            '.' => {
                if self.peek() == Some('.') {
                    self.advance();
                    if self.peek() == Some('=') {
                        self.advance();
                        TokenKind::DotDotEq
                    } else {
                        TokenKind::DotDot
                    }
                } else {
                    TokenKind::Dot
                }
            }
            '(' => TokenKind::LParen,
            ')' => TokenKind::RParen,
            '{' => TokenKind::LBrace,
            '}' => TokenKind::RBrace,
            '[' => TokenKind::LBracket,
            ']' => TokenKind::RBracket,
            '|' => TokenKind::Pipe,
            '$' => TokenKind::Dollar,
            '?' => TokenKind::Question,
            _ => {
                let span = self.make_span(start);
                self.diagnostics.push(
                    Diagnostic::error(format!("unexpected character: '{}'", ch))
                        .with_code(ErrorCode::UNEXPECTED_CHAR)
                        .with_label(span, "unrecognized character"),
                );
                TokenKind::Error(format!("unexpected character: {}", ch))
            }
        };

        Token::new(kind, self.make_span(start))
    }

    /// Tokenizes the entire source code
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        loop {
            let token = self.next_token();
            let is_eof = token.kind == TokenKind::Eof;
            tokens.push(token);

            if is_eof {
                break;
            }
        }

        tokens
    }
}

/// Tokenizes source code and returns the tokens
pub fn tokenize(source: &str, file_id: u32) -> (Vec<Token>, Diagnostics) {
    let mut lexer = Lexer::new(source, file_id);
    let tokens = lexer.tokenize();
    (tokens, lexer.take_diagnostics())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lex(source: &str) -> Vec<TokenKind> {
        let mut lexer = Lexer::new(source, 0);
        lexer
            .tokenize()
            .into_iter()
            .map(|t| t.kind)
            .filter(|k| !matches!(k, TokenKind::Newline | TokenKind::Eof))
            .collect()
    }

    #[test]
    fn test_basic_tokens() {
        let tokens = lex("let x: int = 10");
        assert_eq!(
            tokens,
            vec![
                TokenKind::Let,
                TokenKind::Ident("x".into()),
                TokenKind::Colon,
                TokenKind::IntType,
                TokenKind::Eq,
                TokenKind::IntLit(10),
            ]
        );
    }

    #[test]
    fn test_keywords() {
        let tokens = lex("fn async await return");
        assert_eq!(
            tokens,
            vec![
                TokenKind::Fn,
                TokenKind::Async,
                TokenKind::Await,
                TokenKind::Return,
            ]
        );
    }

    #[test]
    fn test_http_keywords() {
        let tokens = lex("api GET POST server middleware");
        assert_eq!(
            tokens,
            vec![
                TokenKind::Api,
                TokenKind::Get,
                TokenKind::Post,
                TokenKind::Server,
                TokenKind::Middleware,
            ]
        );
    }

    #[test]
    fn test_numbers() {
        let tokens = lex("42 3.14 0xFF 0b1010");
        assert_eq!(
            tokens,
            vec![
                TokenKind::IntLit(42),
                TokenKind::FloatLit(3.14),
                TokenKind::IntLit(255),
                TokenKind::IntLit(10),
            ]
        );
    }

    #[test]
    fn test_strings() {
        let tokens = lex(r#""hello" "world\n""#);
        assert_eq!(
            tokens,
            vec![
                TokenKind::StringLit("hello".into()),
                TokenKind::StringLit("world\n".into()),
            ]
        );
    }

    #[test]
    fn test_operators() {
        let tokens = lex("+ - * / == != <= >= -> &mut");
        assert_eq!(
            tokens,
            vec![
                TokenKind::Plus,
                TokenKind::Minus,
                TokenKind::Star,
                TokenKind::Slash,
                TokenKind::EqEq,
                TokenKind::Ne,
                TokenKind::Le,
                TokenKind::Ge,
                TokenKind::Arrow,
                TokenKind::AmpersandMut,
            ]
        );
    }

    #[test]
    fn test_indentation() {
        let source = "if x:\n    y\n    z\nw";
        let mut lexer = Lexer::new(source, 0);
        let tokens: Vec<_> = lexer.tokenize().into_iter().map(|t| t.kind).collect();

        assert!(tokens.contains(&TokenKind::Indent));
        assert!(tokens.contains(&TokenKind::Dedent));
    }

    #[test]
    fn test_borrow() {
        let tokens = lex("&user &mut user");
        assert_eq!(
            tokens,
            vec![
                TokenKind::Ampersand,
                TokenKind::Ident("user".into()),
                TokenKind::AmpersandMut,
                TokenKind::Ident("user".into()),
            ]
        );
    }
}
