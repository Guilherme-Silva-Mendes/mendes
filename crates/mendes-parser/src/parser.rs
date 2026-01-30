//! Parser for the Mendes language
//!
//! Converts a sequence of tokens into AST using recursive descent.

use crate::ast::*;
use mendes_error::{Diagnostic, Diagnostics, ErrorCode, Span};
use mendes_lexer::{Token, TokenKind};

/// Parser for the Mendes language
pub struct Parser {
    /// Tokens to be parsed
    tokens: Vec<Token>,
    /// Current position
    pos: usize,
    /// Accumulated diagnostics
    diagnostics: Diagnostics,
}

impl Parser {
    /// Creates a new parser
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            pos: 0,
            diagnostics: Diagnostics::new(),
        }
    }

    /// Returns the diagnostics
    pub fn diagnostics(&self) -> &Diagnostics {
        &self.diagnostics
    }

    /// Consumes and returns the diagnostics
    pub fn take_diagnostics(&mut self) -> Diagnostics {
        std::mem::take(&mut self.diagnostics)
    }

    // =========================================
    // Helpers
    // =========================================

    /// Returns the current token without advancing
    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or_else(|| {
            self.tokens.last().expect("tokens should not be empty")
        })
    }

    /// Returns the next token without advancing
    fn peek_next(&self) -> &Token {
        self.tokens.get(self.pos + 1).unwrap_or_else(|| {
            self.tokens.last().expect("tokens should not be empty")
        })
    }

    /// Checks if the current token is of the specified type
    fn check(&self, kind: &TokenKind) -> bool {
        std::mem::discriminant(&self.peek().kind) == std::mem::discriminant(kind)
    }

    /// Checks if we've reached the end
    fn is_at_end(&self) -> bool {
        matches!(self.peek().kind, TokenKind::Eof)
    }

    /// Advances to the next token
    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.pos += 1;
        }
        self.previous()
    }

    /// Returns the previous token
    fn previous(&self) -> &Token {
        &self.tokens[self.pos.saturating_sub(1)]
    }

    /// Consumes the token if it matches the expected type
    fn match_token(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Requires a specific token or reports an error
    fn expect(&mut self, kind: &TokenKind, message: &str) -> Result<&Token, ()> {
        if self.check(kind) {
            Ok(self.advance())
        } else {
            self.error_at_current(message);
            Err(())
        }
    }

    /// Skips newlines
    fn skip_newlines(&mut self) {
        while self.check(&TokenKind::Newline) {
            self.advance();
        }
    }

    /// Reports an error at the current token
    fn error_at_current(&mut self, message: &str) {
        let span = self.peek().span;
        self.diagnostics.push(
            Diagnostic::error(message)
                .with_code(ErrorCode::UNEXPECTED_TOKEN)
                .with_label(span, format!("found: {}", self.peek().kind)),
        );
    }

    /// Synchronizes after an error (panic mode recovery)
    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            // Synchronize at newline followed by a statement keyword
            if matches!(self.previous().kind, TokenKind::Newline) {
                match self.peek().kind {
                    TokenKind::Let
                    | TokenKind::Fn
                    | TokenKind::Struct
                    | TokenKind::Enum
                    | TokenKind::Api
                    | TokenKind::Ws
                    | TokenKind::Server
                    | TokenKind::Middleware
                    | TokenKind::Db
                    | TokenKind::If
                    | TokenKind::For
                    | TokenKind::While
                    | TokenKind::Return => return,
                    _ => {}
                }
            }
            self.advance();
        }
    }

    // =========================================
    // Main parsing
    // =========================================

    /// Parses the complete program
    pub fn parse(&mut self) -> Program {
        let mut statements = Vec::new();

        self.skip_newlines();

        while !self.is_at_end() {
            match self.parse_statement() {
                Ok(stmt) => statements.push(stmt),
                Err(_) => self.synchronize(),
            }
            self.skip_newlines();
        }

        Program { statements }
    }

    /// Parses a statement
    fn parse_statement(&mut self) -> Result<Stmt, ()> {
        self.skip_newlines();

        match &self.peek().kind {
            TokenKind::Import => self.parse_import(),
            TokenKind::From => self.parse_from_import(),
            TokenKind::Module => self.parse_module(),
            TokenKind::Let => self.parse_let(),
            TokenKind::Fn => self.parse_fn(),
            TokenKind::Struct => self.parse_struct(),
            TokenKind::Enum => self.parse_enum(),
            TokenKind::Trait => self.parse_trait(),
            TokenKind::Impl => self.parse_impl(),
            TokenKind::Type => self.parse_type_alias(),
            TokenKind::Api => self.parse_api(),
            TokenKind::Ws => self.parse_websocket(),
            TokenKind::Server => self.parse_server(),
            TokenKind::Middleware => self.parse_middleware(),
            TokenKind::Db => self.parse_db(),
            TokenKind::If => self.parse_if(),
            TokenKind::For => self.parse_for(),
            TokenKind::While => self.parse_while(),
            TokenKind::Return => self.parse_return(),
            TokenKind::Break => self.parse_break(),
            TokenKind::Continue => self.parse_continue(),
            _ => self.parse_expr_stmt(),
        }
    }

    /// Parse: `import "path/to/file.ms"` or `import module_name [as alias]`
    fn parse_import(&mut self) -> Result<Stmt, ()> {
        let start_span = self.peek().span;
        self.advance(); // consume 'import'

        let path = if matches!(self.peek().kind, TokenKind::StringLit(_)) {
            // import "path/to/file.ms"
            if let TokenKind::StringLit(s) = &self.peek().kind {
                let p = s.clone();
                self.advance();
                p
            } else {
                unreachable!()
            }
        } else {
            // import module_name
            self.parse_identifier()?
        };

        // Optional alias: import foo as bar
        let alias = if self.match_token(&TokenKind::As) {
            Some(self.parse_identifier()?)
        } else {
            None
        };

        let span = start_span.merge(self.previous().span);
        self.expect_newline()?;

        Ok(Stmt::Import { path, alias, span })
    }

    /// Parse: `from module import item1, item2` or `from module import *`
    fn parse_from_import(&mut self) -> Result<Stmt, ()> {
        let start_span = self.peek().span;
        self.advance(); // consume 'from'

        // Parse module name (can be path with dots: std.collections)
        let module = self.parse_module_path()?;

        // Expect 'import' keyword
        if !self.match_token(&TokenKind::Import) {
            let span = self.peek().span;
            self.diagnostics.push(
                Diagnostic::error("expected 'import' after module name in from statement")
                    .with_code(ErrorCode::UNEXPECTED_TOKEN)
                    .with_label(span, format!("found: {}", self.peek().kind)),
            );
            return Err(());
        }

        // Parse imported items
        let items = if self.match_token(&TokenKind::Star) {
            // from module import *
            ImportItems::All
        } else {
            // from module import item1, item2, ...
            let mut items = Vec::new();

            loop {
                let item_span = self.peek().span;
                let name = self.parse_identifier()?;

                // Optional alias: item as alias
                let alias = if self.match_token(&TokenKind::As) {
                    Some(self.parse_identifier()?)
                } else {
                    None
                };

                let item_end_span = self.previous().span;
                items.push(ImportItem {
                    name,
                    alias,
                    span: item_span.merge(item_end_span),
                });

                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }

            ImportItems::Names(items)
        };

        let span = start_span.merge(self.previous().span);
        self.expect_newline()?;

        Ok(Stmt::FromImport { module, items, span })
    }

    /// Parse module path: `std.collections` or `mymodule`
    fn parse_module_path(&mut self) -> Result<String, ()> {
        let mut path = self.parse_identifier()?;

        while self.match_token(&TokenKind::Dot) {
            path.push('.');
            path.push_str(&self.parse_identifier()?);
        }

        Ok(path)
    }

    /// Parse: `module name`
    fn parse_module(&mut self) -> Result<Stmt, ()> {
        let span = self.peek().span;
        self.advance(); // consume 'module'
        let _name = self.parse_identifier()?;
        self.expect_newline()?;
        // Module is ignored for now (only for compatibility)
        // We return an empty statement - we'll create a type for this
        Ok(Stmt::Expr(Expr::None(span)))
    }

    // =========================================
    // Statements
    // =========================================

    /// Parse: `let [mut] name [: type] = expr`
    fn parse_let(&mut self) -> Result<Stmt, ()> {
        let start_span = self.peek().span;
        self.advance(); // consume 'let'

        let mutable = self.match_token(&TokenKind::Mut);

        let name = self.parse_identifier()?;

        // Optional type
        let ty = if self.match_token(&TokenKind::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };

        self.expect(&TokenKind::Eq, "expected '=' after variable name")?;

        let value = self.parse_expression()?;

        let span = start_span.merge(self.previous().span);
        self.expect_newline()?;

        Ok(Stmt::Let {
            name,
            ty,
            value,
            mutable,
            span,
        })
    }

    /// Parse: `fn name<T, U>(params) -> type [async]:`
    fn parse_fn(&mut self) -> Result<Stmt, ()> {
        let start_span = self.peek().span;
        let is_pub = false; // TODO: implement pub

        self.advance(); // consume 'fn'

        let name = self.parse_identifier()?;

        // Optional generic parameters
        let generic_params = if self.check(&TokenKind::Lt) {
            self.parse_generic_params()?
        } else {
            Vec::new()
        };

        self.expect(&TokenKind::LParen, "expected '(' after function name")?;
        let params = self.parse_param_list()?;
        self.expect(&TokenKind::RParen, "expected ')' after parameters")?;

        // Optional return type
        let return_type = if self.match_token(&TokenKind::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };

        let is_async = self.match_token(&TokenKind::Async);

        self.expect(&TokenKind::Colon, "expected ':' after function signature")?;
        self.expect_newline()?;

        let body = self.parse_block()?;

        let span = start_span.merge(self.previous().span);

        Ok(Stmt::Fn(FnDecl {
            name,
            generic_params,
            params,
            return_type,
            is_async,
            is_pub,
            body,
            span,
        }))
    }

    /// Parse: `struct Name<T, U> [copy]:`
    fn parse_struct(&mut self) -> Result<Stmt, ()> {
        let start_span = self.peek().span;
        self.advance(); // consume 'struct'

        let name = self.parse_identifier()?;

        // Optional generic parameters
        let generic_params = if self.check(&TokenKind::Lt) {
            self.parse_generic_params()?
        } else {
            Vec::new()
        };

        let is_copy = self.match_token(&TokenKind::Copy);

        self.expect(&TokenKind::Colon, "expected ':' after struct name")?;
        self.expect_newline()?;

        let (fields, methods) = self.parse_struct_body()?;

        let span = start_span.merge(self.previous().span);

        Ok(Stmt::Struct(StructDecl {
            name,
            generic_params,
            fields,
            methods,
            is_copy,
            span,
        }))
    }

    /// Parse: `enum Name:`
    fn parse_enum(&mut self) -> Result<Stmt, ()> {
        let start_span = self.peek().span;
        self.advance(); // consume 'enum'

        let name = self.parse_identifier()?;

        self.expect(&TokenKind::Colon, "expected ':' after enum name")?;
        self.expect_newline()?;

        let variants = self.parse_enum_variants()?;

        let span = start_span.merge(self.previous().span);

        Ok(Stmt::Enum(EnumDecl {
            name,
            variants,
            span,
        }))
    }

    /// Parse enum variants
    fn parse_enum_variants(&mut self) -> Result<Vec<EnumVariant>, ()> {
        self.expect(&TokenKind::Indent, "expected indented block for enum")?;

        let mut variants = Vec::new();

        while !self.check(&TokenKind::Dedent) && !self.is_at_end() {
            self.skip_newlines();
            if self.check(&TokenKind::Dedent) {
                break;
            }

            let variant_span = self.peek().span;
            let variant_name = self.parse_identifier()?;

            // Check for associated data
            let data = if self.match_token(&TokenKind::LParen) {
                // Tuple variant: VariantName(Type1, Type2, ...)
                let mut types = Vec::new();
                if !self.check(&TokenKind::RParen) {
                    types.push(self.parse_type()?);
                    while self.match_token(&TokenKind::Comma) {
                        types.push(self.parse_type()?);
                    }
                }
                self.expect(&TokenKind::RParen, "expected ')' after variant types")?;
                EnumVariantData::Tuple(types)
            } else if self.match_token(&TokenKind::LBrace) {
                // Struct variant: VariantName { field: Type, ... }
                let mut fields = Vec::new();
                if !self.check(&TokenKind::RBrace) {
                    loop {
                        let field_span = self.peek().span;
                        let field_name = self.parse_identifier()?;
                        self.expect(&TokenKind::Colon, "expected ':' after field name")?;
                        let field_type = self.parse_type()?;
                        fields.push(Field {
                            name: field_name,
                            ty: field_type,
                            span: field_span.merge(self.previous().span),
                        });
                        if !self.match_token(&TokenKind::Comma) {
                            break;
                        }
                    }
                }
                self.expect(&TokenKind::RBrace, "expected '}' after variant fields")?;
                EnumVariantData::Struct(fields)
            } else {
                // Unit variant: just the name
                EnumVariantData::Unit
            };

            variants.push(EnumVariant {
                name: variant_name,
                data,
                span: variant_span.merge(self.previous().span),
            });

            self.skip_newlines();
        }

        self.match_token(&TokenKind::Dedent);

        Ok(variants)
    }

    /// Parse: `api METHOD /path [async]:`
    fn parse_api(&mut self) -> Result<Stmt, ()> {
        let start_span = self.peek().span;
        self.advance(); // consume 'api'

        let method = self.parse_http_method()?;
        let path = self.parse_path_pattern()?;
        let is_async = self.match_token(&TokenKind::Async);

        self.expect(&TokenKind::Colon, "expected ':' after API path")?;
        self.expect_newline()?;

        // Parse directives and body
        let (middlewares, body_type, return_type, handler) = self.parse_api_body()?;

        let span = start_span.merge(self.previous().span);

        Ok(Stmt::Api(ApiDecl {
            method,
            path,
            is_async,
            middlewares,
            body_type,
            return_type,
            handler,
            span,
        }))
    }

    /// Parse: `ws /path:`
    fn parse_websocket(&mut self) -> Result<Stmt, ()> {
        let start_span = self.peek().span;
        self.advance(); // consume 'ws'

        let path = self.parse_path_pattern()?;

        self.expect(&TokenKind::Colon, "expected ':' after WebSocket path")?;
        self.expect_newline()?;
        self.expect(&TokenKind::Indent, "expected indented block for WebSocket")?;

        let mut middlewares = Vec::new();
        let mut on_connect = None;
        let mut on_message = None;
        let mut on_disconnect = None;

        while !self.check(&TokenKind::Dedent) && !self.is_at_end() {
            self.skip_newlines();
            if self.check(&TokenKind::Dedent) {
                break;
            }

            // Parse directives and event handlers
            if let TokenKind::Ident(name) = &self.peek().kind.clone() {
                match name.as_str() {
                    "use" => {
                        self.advance();
                        let middleware_name = self.parse_identifier()?;
                        middlewares.push(middleware_name);
                        self.skip_newlines();
                    }
                    "on_connect" => {
                        self.advance();
                        self.expect(&TokenKind::Colon, "expected ':' after 'on_connect'")?;
                        self.expect_newline()?;
                        on_connect = Some(self.parse_block()?);
                    }
                    "on_message" => {
                        self.advance();
                        self.expect(&TokenKind::Colon, "expected ':' after 'on_message'")?;
                        self.expect_newline()?;
                        on_message = Some(self.parse_block()?);
                    }
                    "on_disconnect" => {
                        self.advance();
                        self.expect(&TokenKind::Colon, "expected ':' after 'on_disconnect'")?;
                        self.expect_newline()?;
                        on_disconnect = Some(self.parse_block()?);
                    }
                    _ => {
                        self.error_at_current(&format!("unexpected WebSocket directive: {}", name));
                        self.advance();
                    }
                }
            } else {
                self.error_at_current("expected WebSocket event handler (on_connect, on_message, on_disconnect)");
                self.advance();
            }
        }

        self.match_token(&TokenKind::Dedent);

        let span = start_span.merge(self.previous().span);

        Ok(Stmt::WebSocket(WsDecl {
            path,
            middlewares,
            on_connect,
            on_message,
            on_disconnect,
            span,
        }))
    }

    /// Parse: `server:`
    fn parse_server(&mut self) -> Result<Stmt, ()> {
        let start_span = self.peek().span;
        self.advance(); // consume 'server'

        self.expect(&TokenKind::Colon, "expected ':' after 'server'")?;
        self.expect_newline()?;
        self.expect(&TokenKind::Indent, "expected indented block")?;

        let mut host = String::from("0.0.0.0");
        let mut port = 8080u16;

        while !self.check(&TokenKind::Dedent) && !self.is_at_end() {
            self.skip_newlines();
            if self.check(&TokenKind::Dedent) {
                break;
            }

            if let TokenKind::Ident(name) = &self.peek().kind.clone() {
                self.advance();
                match name.as_str() {
                    "host" => {
                        if let TokenKind::StringLit(s) = &self.peek().kind.clone() {
                            host = s.clone();
                            self.advance();
                        }
                    }
                    "port" => {
                        if let TokenKind::IntLit(p) = &self.peek().kind {
                            port = *p as u16;
                            self.advance();
                        }
                    }
                    _ => {}
                }
            }
            self.skip_newlines();
        }

        self.match_token(&TokenKind::Dedent);

        let span = start_span.merge(self.previous().span);

        Ok(Stmt::Server(ServerDecl { host, port, span }))
    }

    /// Parse: `middleware name:`
    fn parse_middleware(&mut self) -> Result<Stmt, ()> {
        let start_span = self.peek().span;
        self.advance(); // consume 'middleware'

        let name = self.parse_identifier()?;

        self.expect(&TokenKind::Colon, "expected ':' after middleware name")?;
        self.expect_newline()?;

        let body = self.parse_block()?;

        let span = start_span.merge(self.previous().span);

        Ok(Stmt::Middleware(MiddlewareDecl { name, body, span }))
    }

    /// Parse: `db type name:`
    fn parse_db(&mut self) -> Result<Stmt, ()> {
        let start_span = self.peek().span;
        self.advance(); // consume 'db'

        let db_type = self.parse_db_type()?;
        let name = self.parse_identifier()?;

        self.expect(&TokenKind::Colon, "expected ':' after database name")?;
        self.expect_newline()?;
        self.expect(&TokenKind::Indent, "expected indented block")?;

        let mut url = String::new();
        let mut pool_size = 10u32;

        while !self.check(&TokenKind::Dedent) && !self.is_at_end() {
            self.skip_newlines();
            if self.check(&TokenKind::Dedent) {
                break;
            }

            if let TokenKind::Ident(field_name) = &self.peek().kind.clone() {
                self.advance();
                match field_name.as_str() {
                    "url" => {
                        if let TokenKind::StringLit(s) = &self.peek().kind.clone() {
                            url = s.clone();
                            self.advance();
                        }
                    }
                    "pool" => {
                        if let TokenKind::IntLit(p) = &self.peek().kind {
                            pool_size = *p as u32;
                            self.advance();
                        }
                    }
                    _ => {}
                }
            }
            self.skip_newlines();
        }

        self.match_token(&TokenKind::Dedent);

        let span = start_span.merge(self.previous().span);

        Ok(Stmt::Db(DbDecl {
            db_type,
            name,
            url,
            pool_size,
            span,
        }))
    }

    /// Parse: `if expr:`
    fn parse_if(&mut self) -> Result<Stmt, ()> {
        let start_span = self.peek().span;
        self.advance(); // consume 'if'

        let condition = self.parse_expression()?;

        self.expect(&TokenKind::Colon, "expected ':' after condition")?;
        self.expect_newline()?;

        let then_block = self.parse_block()?;

        let else_block = if self.match_token(&TokenKind::Else) {
            self.expect(&TokenKind::Colon, "expected ':' after 'else'")?;
            self.expect_newline()?;
            Some(self.parse_block()?)
        } else {
            None
        };

        let span = start_span.merge(self.previous().span);

        Ok(Stmt::If {
            condition,
            then_block,
            else_block,
            span,
        })
    }

    /// Parse: `for var in expr:`
    fn parse_for(&mut self) -> Result<Stmt, ()> {
        let start_span = self.peek().span;
        self.advance(); // consume 'for'

        let var = self.parse_identifier()?;

        self.expect(&TokenKind::In, "expected 'in' after variable")?;

        let iter = self.parse_expression()?;

        self.expect(&TokenKind::Colon, "expected ':' after expression")?;
        self.expect_newline()?;

        let body = self.parse_block()?;

        let span = start_span.merge(self.previous().span);

        Ok(Stmt::For {
            var,
            iter,
            body,
            span,
        })
    }

    /// Parse: `while expr:`
    fn parse_while(&mut self) -> Result<Stmt, ()> {
        let start_span = self.peek().span;
        self.advance(); // consume 'while'

        let condition = self.parse_expression()?;

        self.expect(&TokenKind::Colon, "expected ':' after condition")?;
        self.expect_newline()?;

        let body = self.parse_block()?;

        let span = start_span.merge(self.previous().span);

        Ok(Stmt::While {
            condition,
            body,
            span,
        })
    }

    /// Parse: `return [expr]`
    fn parse_return(&mut self) -> Result<Stmt, ()> {
        let start_span = self.peek().span;
        self.advance(); // consume 'return'

        let value = if !self.check(&TokenKind::Newline) && !self.check(&TokenKind::Dedent) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        let span = start_span.merge(self.previous().span);
        self.expect_newline()?;

        Ok(Stmt::Return { value, span })
    }

    /// Parse: `break`
    fn parse_break(&mut self) -> Result<Stmt, ()> {
        let span = self.peek().span;
        self.advance(); // consume 'break'
        self.expect_newline()?;
        Ok(Stmt::Break { span })
    }

    /// Parse: `continue`
    fn parse_continue(&mut self) -> Result<Stmt, ()> {
        let span = self.peek().span;
        self.advance(); // consume 'continue'
        self.expect_newline()?;
        Ok(Stmt::Continue { span })
    }

    /// Parse: `type Name = Type`
    fn parse_type_alias(&mut self) -> Result<Stmt, ()> {
        let start_span = self.peek().span;
        self.advance(); // consume 'type'

        let name = self.parse_identifier()?;
        self.expect(&TokenKind::Eq, "expected '=' after type name")?;
        let ty = self.parse_type()?;

        let span = start_span.merge(self.previous().span);
        self.expect_newline()?;

        Ok(Stmt::TypeAlias { name, ty, span })
    }

    /// Parse: `trait Name:`
    fn parse_trait(&mut self) -> Result<Stmt, ()> {
        let start_span = self.peek().span;
        self.advance(); // consume 'trait'

        let name = self.parse_identifier()?;

        // Optional generic parameters
        let generic_params = if self.check(&TokenKind::Lt) {
            self.parse_generic_params()?
        } else {
            Vec::new()
        };

        self.expect(&TokenKind::Colon, "expected ':' after trait name")?;
        self.expect_newline()?;

        let methods = self.parse_trait_body()?;

        let span = start_span.merge(self.previous().span);

        Ok(Stmt::Trait(TraitDecl {
            name,
            generic_params,
            methods,
            span,
        }))
    }

    /// Parse trait body (method signatures without bodies)
    fn parse_trait_body(&mut self) -> Result<Vec<TraitMethod>, ()> {
        self.expect(&TokenKind::Indent, "expected indented block for trait")?;

        let mut methods = Vec::new();

        while !self.check(&TokenKind::Dedent) && !self.is_at_end() {
            self.skip_newlines();
            if self.check(&TokenKind::Dedent) {
                break;
            }

            methods.push(self.parse_trait_method()?);
            self.skip_newlines();
        }

        self.match_token(&TokenKind::Dedent);

        Ok(methods)
    }

    /// Parse trait method signature
    fn parse_trait_method(&mut self) -> Result<TraitMethod, ()> {
        let start_span = self.peek().span;

        self.expect(&TokenKind::Fn, "expected 'fn' for trait method")?;

        let name = self.parse_identifier()?;

        self.expect(&TokenKind::LParen, "expected '(' after method name")?;

        // Parse receiver
        let receiver = self.parse_method_receiver()?;

        // Parse remaining parameters
        let mut params = Vec::new();
        if self.match_token(&TokenKind::Comma) {
            if !self.check(&TokenKind::RParen) {
                params = self.parse_param_list()?;
            }
        }

        self.expect(&TokenKind::RParen, "expected ')' after parameters")?;

        // Optional return type
        let return_type = if self.match_token(&TokenKind::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };

        let is_async = self.match_token(&TokenKind::Async);

        let span = start_span.merge(self.previous().span);
        self.expect_newline()?;

        Ok(TraitMethod {
            name,
            params,
            return_type,
            is_async,
            receiver,
            span,
        })
    }

    /// Parse: `impl TraitName for TypeName:`
    fn parse_impl(&mut self) -> Result<Stmt, ()> {
        let start_span = self.peek().span;
        self.advance(); // consume 'impl'

        let trait_name = self.parse_identifier()?;

        // Optional generic parameters
        let generic_params = if self.check(&TokenKind::Lt) {
            self.parse_generic_params()?
        } else {
            Vec::new()
        };

        self.expect(&TokenKind::For, "expected 'for' after trait name")?;
        let type_name = self.parse_identifier()?;

        self.expect(&TokenKind::Colon, "expected ':' after type name")?;
        self.expect_newline()?;

        let methods = self.parse_impl_body()?;

        let span = start_span.merge(self.previous().span);

        Ok(Stmt::ImplTrait(ImplTraitDecl {
            trait_name,
            type_name,
            generic_params,
            methods,
            span,
        }))
    }

    /// Parse impl body (methods with bodies)
    fn parse_impl_body(&mut self) -> Result<Vec<MethodDecl>, ()> {
        self.expect(&TokenKind::Indent, "expected indented block for impl")?;

        let mut methods = Vec::new();

        while !self.check(&TokenKind::Dedent) && !self.is_at_end() {
            self.skip_newlines();
            if self.check(&TokenKind::Dedent) {
                break;
            }

            methods.push(self.parse_method()?);
            self.skip_newlines();
        }

        self.match_token(&TokenKind::Dedent);

        Ok(methods)
    }

    /// Parse generic parameters: `<T, U: Trait>`
    fn parse_generic_params(&mut self) -> Result<Vec<GenericParam>, ()> {
        self.expect(&TokenKind::Lt, "expected '<'")?;

        let mut params = Vec::new();

        if !self.check(&TokenKind::Gt) {
            loop {
                let span = self.peek().span;
                let name = self.parse_identifier()?;

                // Optional bounds: T: Trait1 + Trait2
                let mut bounds = Vec::new();
                if self.match_token(&TokenKind::Colon) {
                    bounds.push(self.parse_identifier()?);
                    while self.match_token(&TokenKind::Plus) {
                        bounds.push(self.parse_identifier()?);
                    }
                }

                params.push(GenericParam {
                    name,
                    bounds,
                    span: span.merge(self.previous().span),
                });

                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
        }

        self.expect(&TokenKind::Gt, "expected '>'")?;

        Ok(params)
    }

    /// Parse expression statement
    fn parse_expr_stmt(&mut self) -> Result<Stmt, ()> {
        let expr = self.parse_expression()?;
        self.expect_newline()?;
        Ok(Stmt::Expr(expr))
    }

    // =========================================
    // Blocks
    // =========================================

    /// Parse indented block
    fn parse_block(&mut self) -> Result<Vec<Stmt>, ()> {
        self.expect(&TokenKind::Indent, "expected indented block")?;

        let mut statements = Vec::new();

        while !self.check(&TokenKind::Dedent) && !self.is_at_end() {
            self.skip_newlines();
            if self.check(&TokenKind::Dedent) {
                break;
            }

            match self.parse_statement() {
                Ok(stmt) => statements.push(stmt),
                Err(_) => self.synchronize(),
            }
        }

        self.match_token(&TokenKind::Dedent);

        Ok(statements)
    }

    /// Parse struct body (fields and methods)
    fn parse_struct_body(&mut self) -> Result<(Vec<Field>, Vec<MethodDecl>), ()> {
        self.expect(&TokenKind::Indent, "expected indented block")?;

        let mut fields = Vec::new();
        let mut methods = Vec::new();

        while !self.check(&TokenKind::Dedent) && !self.is_at_end() {
            self.skip_newlines();
            if self.check(&TokenKind::Dedent) {
                break;
            }

            // Check if it's a method (starts with 'fn')
            if self.check(&TokenKind::Fn) {
                methods.push(self.parse_method()?);
            } else {
                // Parse field
                let span = self.peek().span;
                let name = self.parse_identifier()?;
                self.expect(&TokenKind::Colon, "expected ':' after field name")?;
                let ty = self.parse_type()?;

                fields.push(Field {
                    name,
                    ty,
                    span: span.merge(self.previous().span),
                });
            }

            self.skip_newlines();
        }

        self.match_token(&TokenKind::Dedent);

        Ok((fields, methods))
    }

    /// Parse method declaration inside a struct
    fn parse_method(&mut self) -> Result<MethodDecl, ()> {
        let start_span = self.peek().span;
        let is_pub = false; // TODO: implement pub

        self.advance(); // consume 'fn'

        let name = self.parse_identifier()?;

        self.expect(&TokenKind::LParen, "expected '(' after method name")?;

        // Parse receiver (self, &self, or &mut self)
        let receiver = self.parse_method_receiver()?;

        // Parse remaining parameters
        let mut params = Vec::new();
        if receiver != MethodReceiver::Value || self.check(&TokenKind::Comma) {
            // If we have a receiver, skip comma before additional params
            if self.match_token(&TokenKind::Comma) || receiver == MethodReceiver::Value {
                if !self.check(&TokenKind::RParen) {
                    params = self.parse_param_list()?;
                }
            }
        }

        self.expect(&TokenKind::RParen, "expected ')' after parameters")?;

        // Optional return type
        let return_type = if self.match_token(&TokenKind::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };

        let is_async = self.match_token(&TokenKind::Async);

        self.expect(&TokenKind::Colon, "expected ':' after method signature")?;
        self.expect_newline()?;

        let body = self.parse_block()?;

        let span = start_span.merge(self.previous().span);

        Ok(MethodDecl {
            name,
            params,
            return_type,
            is_async,
            is_pub,
            receiver,
            body,
            span,
        })
    }

    /// Parse method receiver: self, &self, or &mut self
    fn parse_method_receiver(&mut self) -> Result<MethodReceiver, ()> {
        if self.match_token(&TokenKind::AmpersandMut) {
            // &mut self
            if let TokenKind::Ident(name) = &self.peek().kind {
                if name == "self" {
                    self.advance();
                    return Ok(MethodReceiver::MutRef);
                }
            }
            self.error_at_current("expected 'self' after '&mut'");
            return Err(());
        }

        if self.match_token(&TokenKind::Ampersand) {
            // &self
            if let TokenKind::Ident(name) = &self.peek().kind {
                if name == "self" {
                    self.advance();
                    return Ok(MethodReceiver::Ref);
                }
            }
            self.error_at_current("expected 'self' after '&'");
            return Err(());
        }

        if let TokenKind::Ident(name) = &self.peek().kind {
            if name == "self" {
                self.advance();
                return Ok(MethodReceiver::Value);
            }
        }

        // No receiver - this is a static method, default to Ref for simplicity
        // In a real implementation, we'd have a separate variant for static methods
        Ok(MethodReceiver::Ref)
    }

    /// Parse match expression: `match expr:`
    fn parse_match_expr(&mut self) -> Result<Expr, ()> {
        let start_span = self.peek().span;
        self.advance(); // consume 'match'

        let expr = self.parse_expression()?;

        self.expect(&TokenKind::Colon, "expected ':' after match expression")?;
        self.expect_newline()?;

        let arms = self.parse_match_arms()?;

        let span = start_span.merge(self.previous().span);

        Ok(Expr::Match {
            expr: Box::new(expr),
            arms,
            span,
        })
    }

    /// Parse match arms
    fn parse_match_arms(&mut self) -> Result<Vec<MatchArm>, ()> {
        self.expect(&TokenKind::Indent, "expected indented block for match")?;

        let mut arms = Vec::new();

        while !self.check(&TokenKind::Dedent) && !self.is_at_end() {
            self.skip_newlines();
            if self.check(&TokenKind::Dedent) {
                break;
            }

            let arm_span = self.peek().span;
            let pattern = self.parse_pattern()?;

            // Optional guard: `if condition`
            let guard = if self.match_token(&TokenKind::If) {
                Some(self.parse_expression()?)
            } else {
                None
            };

            self.expect(&TokenKind::Colon, "expected ':' after pattern")?;
            self.expect_newline()?;

            let body = self.parse_block()?;

            arms.push(MatchArm {
                pattern,
                guard,
                body,
                span: arm_span.merge(self.previous().span),
            });
        }

        self.match_token(&TokenKind::Dedent);

        Ok(arms)
    }

    /// Parse a pattern
    fn parse_pattern(&mut self) -> Result<Pattern, ()> {
        self.parse_or_pattern()
    }

    /// Parse or pattern: `A | B | C`
    fn parse_or_pattern(&mut self) -> Result<Pattern, ()> {
        let mut left = self.parse_primary_pattern()?;

        if self.check(&TokenKind::Pipe) {
            let start_span = left.span();
            let mut patterns = vec![left];

            while self.match_token(&TokenKind::Pipe) {
                patterns.push(self.parse_primary_pattern()?);
            }

            let end_span = patterns.last().map(|p| p.span()).unwrap_or(start_span);
            left = Pattern::Or(patterns, start_span.merge(end_span));
        }

        Ok(left)
    }

    /// Parse primary pattern
    fn parse_primary_pattern(&mut self) -> Result<Pattern, ()> {
        let token = self.peek().clone();

        match &token.kind {
            // Wildcard: _
            TokenKind::Ident(name) if name == "_" => {
                self.advance();
                Ok(Pattern::Wildcard(token.span))
            }

            // Literal patterns
            TokenKind::IntLit(n) => {
                let n = *n;
                self.advance();
                Ok(Pattern::Literal(Expr::IntLit(n, token.span)))
            }
            TokenKind::FloatLit(n) => {
                let n = *n;
                self.advance();
                Ok(Pattern::Literal(Expr::FloatLit(n, token.span)))
            }
            TokenKind::StringLit(s) => {
                let s = s.clone();
                self.advance();
                Ok(Pattern::Literal(Expr::StringLit(s, token.span)))
            }
            TokenKind::True => {
                self.advance();
                Ok(Pattern::Literal(Expr::BoolLit(true, token.span)))
            }
            TokenKind::False => {
                self.advance();
                Ok(Pattern::Literal(Expr::BoolLit(false, token.span)))
            }

            // None pattern
            TokenKind::None => {
                self.advance();
                Ok(Pattern::Variant {
                    enum_name: Some("Option".to_string()),
                    variant: "None".to_string(),
                    data: VariantPatternData::Unit,
                    span: token.span,
                })
            }

            // Some(pattern) pattern
            TokenKind::Some => {
                self.advance();
                self.expect(&TokenKind::LParen, "expected '(' after 'Some'")?;
                let inner = self.parse_pattern()?;
                self.expect(&TokenKind::RParen, "expected ')'")?;
                let span = token.span.merge(self.previous().span);
                Ok(Pattern::Variant {
                    enum_name: Some("Option".to_string()),
                    variant: "Some".to_string(),
                    data: VariantPatternData::Tuple(vec![inner]),
                    span,
                })
            }

            // Ok(pattern) pattern
            TokenKind::Ok => {
                self.advance();
                self.expect(&TokenKind::LParen, "expected '(' after 'Ok'")?;
                let inner = self.parse_pattern()?;
                self.expect(&TokenKind::RParen, "expected ')'")?;
                let span = token.span.merge(self.previous().span);
                Ok(Pattern::Variant {
                    enum_name: Some("Result".to_string()),
                    variant: "Ok".to_string(),
                    data: VariantPatternData::Tuple(vec![inner]),
                    span,
                })
            }

            // Err(pattern) pattern
            TokenKind::Err => {
                self.advance();
                self.expect(&TokenKind::LParen, "expected '(' after 'Err'")?;
                let inner = self.parse_pattern()?;
                self.expect(&TokenKind::RParen, "expected ')'")?;
                let span = token.span.merge(self.previous().span);
                Ok(Pattern::Variant {
                    enum_name: Some("Result".to_string()),
                    variant: "Err".to_string(),
                    data: VariantPatternData::Tuple(vec![inner]),
                    span,
                })
            }

            // Tuple pattern: (a, b, c)
            TokenKind::LParen => {
                self.advance();
                let mut patterns = Vec::new();
                if !self.check(&TokenKind::RParen) {
                    patterns.push(self.parse_pattern()?);
                    while self.match_token(&TokenKind::Comma) {
                        patterns.push(self.parse_pattern()?);
                    }
                }
                self.expect(&TokenKind::RParen, "expected ')'")?;
                let span = token.span.merge(self.previous().span);
                Ok(Pattern::Tuple(patterns, span))
            }

            // Identifier or variant pattern
            TokenKind::Ident(name) => {
                let name = name.clone();
                self.advance();

                // Check for :: (qualified variant)
                if self.match_token(&TokenKind::ColonColon) {
                    let variant = self.parse_identifier()?;
                    let data = self.parse_variant_pattern_data()?;
                    let span = token.span.merge(self.previous().span);
                    return Ok(Pattern::Variant {
                        enum_name: Some(name),
                        variant,
                        data,
                        span,
                    });
                }

                // Check for { (struct pattern)
                if self.check(&TokenKind::LBrace) {
                    self.advance();
                    let fields = self.parse_struct_pattern_fields()?;
                    self.expect(&TokenKind::RBrace, "expected '}'")?;
                    let span = token.span.merge(self.previous().span);
                    return Ok(Pattern::Struct {
                        name,
                        fields,
                        span,
                    });
                }

                // Check for ( (tuple variant pattern)
                if self.check(&TokenKind::LParen) {
                    let data = self.parse_variant_pattern_data()?;
                    let span = token.span.merge(self.previous().span);
                    return Ok(Pattern::Variant {
                        enum_name: None,
                        variant: name,
                        data,
                        span,
                    });
                }

                // Simple identifier pattern (binds the value)
                // Check for `mut` modifier
                Ok(Pattern::Ident {
                    name,
                    mutable: false,
                    span: token.span,
                })
            }

            // Mutable binding: mut x
            TokenKind::Mut => {
                self.advance();
                let name = self.parse_identifier()?;
                let span = token.span.merge(self.previous().span);
                Ok(Pattern::Ident {
                    name,
                    mutable: true,
                    span,
                })
            }

            _ => {
                self.error_at_current("pattern expected");
                Err(())
            }
        }
    }

    /// Parse variant pattern data
    fn parse_variant_pattern_data(&mut self) -> Result<VariantPatternData, ()> {
        if self.match_token(&TokenKind::LParen) {
            // Tuple variant: Variant(a, b)
            let mut patterns = Vec::new();
            if !self.check(&TokenKind::RParen) {
                patterns.push(self.parse_pattern()?);
                while self.match_token(&TokenKind::Comma) {
                    patterns.push(self.parse_pattern()?);
                }
            }
            self.expect(&TokenKind::RParen, "expected ')'")?;
            Ok(VariantPatternData::Tuple(patterns))
        } else if self.match_token(&TokenKind::LBrace) {
            // Struct variant: Variant { x, y }
            let fields = self.parse_struct_pattern_fields()?;
            self.expect(&TokenKind::RBrace, "expected '}'")?;
            Ok(VariantPatternData::Struct(fields))
        } else {
            // Unit variant
            Ok(VariantPatternData::Unit)
        }
    }

    /// Parse struct pattern fields: `{ x, y: pattern, ... }`
    fn parse_struct_pattern_fields(&mut self) -> Result<Vec<(String, Option<Pattern>)>, ()> {
        let mut fields = Vec::new();

        if !self.check(&TokenKind::RBrace) {
            loop {
                let field_name = self.parse_identifier()?;

                // Check for : pattern (explicit binding)
                let pattern = if self.match_token(&TokenKind::Colon) {
                    Some(self.parse_pattern()?)
                } else {
                    None // shorthand: { x } means { x: x }
                };

                fields.push((field_name, pattern));

                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
        }

        Ok(fields)
    }

    /// Parse API body (directives + handler)
    fn parse_api_body(&mut self) -> Result<(Vec<String>, Option<Type>, Option<Type>, Vec<Stmt>), ()> {
        self.expect(&TokenKind::Indent, "expected indented block")?;

        let mut middlewares = Vec::new();
        let mut body_type = None;
        let mut return_type = None;
        let mut handler = Vec::new();
        let mut parsing_directives = true;

        // Parse directives and handler in a single loop
        while !self.check(&TokenKind::Dedent) && !self.is_at_end() {
            self.skip_newlines();
            if self.check(&TokenKind::Dedent) {
                break;
            }

            // If we're still parsing directives
            if parsing_directives {
                match &self.peek().kind {
                    TokenKind::Use => {
                        self.advance();
                        let name = self.parse_identifier()?;
                        middlewares.push(name);
                        self.skip_newlines();
                        continue;
                    }
                    TokenKind::Body if body_type.is_none() => {
                        self.advance();
                        body_type = Some(self.parse_type()?);
                        self.skip_newlines();
                        continue;
                    }
                    TokenKind::Return if return_type.is_none() && handler.is_empty() => {
                        // Check if it's a directive (return Type) or statement (return expr)
                        // Directive: return followed by type (identifier starting with uppercase or type keyword)
                        self.advance(); // consume return

                        // Try to parse as type
                        let is_type_decl = match &self.peek().kind {
                            TokenKind::IntType | TokenKind::FloatType |
                            TokenKind::BoolType | TokenKind::StringType |
                            TokenKind::ResultType | TokenKind::OptionType => true,
                            TokenKind::Ident(name) => name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false),
                            TokenKind::LBracket | TokenKind::Ampersand | TokenKind::AmpersandMut => true,
                            _ => false,
                        };

                        if is_type_decl {
                            return_type = Some(self.parse_type()?);
                            self.skip_newlines();
                            continue;
                        } else {
                            // It's a return statement, not a directive
                            parsing_directives = false;
                            let value = if !self.check(&TokenKind::Newline) && !self.check(&TokenKind::Dedent) {
                                Some(self.parse_expression()?)
                            } else {
                                None
                            };
                            let span = self.previous().span;
                            self.skip_newlines();
                            handler.push(Stmt::Return { value, span });
                            continue;
                        }
                    }
                    _ => {
                        parsing_directives = false;
                    }
                }
            }

            // Parse handler statement
            match self.parse_statement() {
                Ok(stmt) => handler.push(stmt),
                Err(_) => self.synchronize(),
            }
        }

        self.match_token(&TokenKind::Dedent);

        Ok((middlewares, body_type, return_type, handler))
    }

    // =========================================
    // Expressions (Pratt Parser)
    // =========================================

    /// Parse expression
    fn parse_expression(&mut self) -> Result<Expr, ()> {
        self.parse_assignment()
    }

    /// Parse assignment
    fn parse_assignment(&mut self) -> Result<Expr, ()> {
        let expr = self.parse_range()?;

        if self.check(&TokenKind::Eq)
            || self.check(&TokenKind::PlusEq)
            || self.check(&TokenKind::MinusEq)
            || self.check(&TokenKind::StarEq)
            || self.check(&TokenKind::SlashEq)
        {
            let op_token = self.advance().clone();
            let op = match op_token.kind {
                TokenKind::Eq => BinOp::Assign,
                TokenKind::PlusEq => BinOp::AddAssign,
                TokenKind::MinusEq => BinOp::SubAssign,
                TokenKind::StarEq => BinOp::MulAssign,
                TokenKind::SlashEq => BinOp::DivAssign,
                _ => unreachable!(),
            };

            let value = self.parse_assignment()?;
            let span = expr.span().merge(value.span());

            return Ok(Expr::Binary {
                left: Box::new(expr),
                op,
                right: Box::new(value),
                span,
            });
        }

        Ok(expr)
    }

    /// Parse range expressions: `start..end` or `start..=end`
    fn parse_range(&mut self) -> Result<Expr, ()> {
        let start_span = self.peek().span;

        // Check for range without start: `..end` or `..=end`
        if self.check(&TokenKind::DotDot) || self.check(&TokenKind::DotDotEq) {
            let inclusive = self.check(&TokenKind::DotDotEq);
            self.advance();

            // Check for range without end: `..`
            let end = if self.check(&TokenKind::Newline)
                || self.check(&TokenKind::Dedent)
                || self.check(&TokenKind::Colon)
                || self.check(&TokenKind::RParen)
                || self.check(&TokenKind::RBracket)
                || self.check(&TokenKind::Comma)
            {
                None
            } else {
                Some(Box::new(self.parse_or()?))
            };

            let span = start_span.merge(self.previous().span);
            return Ok(Expr::Range {
                start: None,
                end,
                inclusive,
                span,
            });
        }

        let left = self.parse_or()?;

        // Check for range operator
        if self.check(&TokenKind::DotDot) || self.check(&TokenKind::DotDotEq) {
            let inclusive = self.check(&TokenKind::DotDotEq);
            self.advance();

            // Check for range without end: `start..`
            let end = if self.check(&TokenKind::Newline)
                || self.check(&TokenKind::Dedent)
                || self.check(&TokenKind::Colon)
                || self.check(&TokenKind::RParen)
                || self.check(&TokenKind::RBracket)
                || self.check(&TokenKind::Comma)
            {
                None
            } else {
                Some(Box::new(self.parse_or()?))
            };

            let span = left.span().merge(self.previous().span);
            return Ok(Expr::Range {
                start: Some(Box::new(left)),
                end,
                inclusive,
                span,
            });
        }

        Ok(left)
    }

    /// Parse OR
    fn parse_or(&mut self) -> Result<Expr, ()> {
        let mut left = self.parse_and()?;

        while self.match_token(&TokenKind::Or) {
            let right = self.parse_and()?;
            let span = left.span().merge(right.span());
            left = Expr::Binary {
                left: Box::new(left),
                op: BinOp::Or,
                right: Box::new(right),
                span,
            };
        }

        Ok(left)
    }

    /// Parse AND
    fn parse_and(&mut self) -> Result<Expr, ()> {
        let mut left = self.parse_not()?;

        while self.match_token(&TokenKind::And) {
            let right = self.parse_not()?;
            let span = left.span().merge(right.span());
            left = Expr::Binary {
                left: Box::new(left),
                op: BinOp::And,
                right: Box::new(right),
                span,
            };
        }

        Ok(left)
    }

    /// Parse NOT
    fn parse_not(&mut self) -> Result<Expr, ()> {
        if self.match_token(&TokenKind::Not) {
            let start_span = self.previous().span;
            let expr = self.parse_not()?;
            let span = start_span.merge(expr.span());
            return Ok(Expr::Unary {
                op: UnaryOp::Not,
                expr: Box::new(expr),
                span,
            });
        }

        self.parse_comparison()
    }

    /// Parse comparison
    fn parse_comparison(&mut self) -> Result<Expr, ()> {
        let mut left = self.parse_additive()?;

        while self.check(&TokenKind::EqEq)
            || self.check(&TokenKind::Ne)
            || self.check(&TokenKind::Lt)
            || self.check(&TokenKind::Le)
            || self.check(&TokenKind::Gt)
            || self.check(&TokenKind::Ge)
            || self.check(&TokenKind::Is)
        {
            let op_token = self.advance().clone();
            let op = match op_token.kind {
                TokenKind::EqEq => BinOp::Eq,
                TokenKind::Ne => BinOp::Ne,
                TokenKind::Lt => BinOp::Lt,
                TokenKind::Le => BinOp::Le,
                TokenKind::Gt => BinOp::Gt,
                TokenKind::Ge => BinOp::Ge,
                TokenKind::Is => BinOp::Eq, // is is treated as ==
                _ => unreachable!(),
            };

            let right = self.parse_additive()?;
            let span = left.span().merge(right.span());

            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span,
            };
        }

        Ok(left)
    }

    /// Parse addition/subtraction
    fn parse_additive(&mut self) -> Result<Expr, ()> {
        let mut left = self.parse_multiplicative()?;

        while self.check(&TokenKind::Plus) || self.check(&TokenKind::Minus) {
            let op_token = self.advance().clone();
            let op = match op_token.kind {
                TokenKind::Plus => BinOp::Add,
                TokenKind::Minus => BinOp::Sub,
                _ => unreachable!(),
            };

            let right = self.parse_multiplicative()?;
            let span = left.span().merge(right.span());

            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span,
            };
        }

        Ok(left)
    }

    /// Parse multiplication/division
    fn parse_multiplicative(&mut self) -> Result<Expr, ()> {
        let mut left = self.parse_unary()?;

        while self.check(&TokenKind::Star)
            || self.check(&TokenKind::Slash)
            || self.check(&TokenKind::Percent)
        {
            let op_token = self.advance().clone();
            let op = match op_token.kind {
                TokenKind::Star => BinOp::Mul,
                TokenKind::Slash => BinOp::Div,
                TokenKind::Percent => BinOp::Mod,
                _ => unreachable!(),
            };

            let right = self.parse_unary()?;
            let span = left.span().merge(right.span());

            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span,
            };
        }

        Ok(left)
    }

    /// Parse unary
    fn parse_unary(&mut self) -> Result<Expr, ()> {
        if self.match_token(&TokenKind::Minus) {
            let start_span = self.previous().span;
            let expr = self.parse_unary()?;
            let span = start_span.merge(expr.span());
            return Ok(Expr::Unary {
                op: UnaryOp::Neg,
                expr: Box::new(expr),
                span,
            });
        }

        if self.match_token(&TokenKind::Ampersand) {
            let start_span = self.previous().span;
            let expr = self.parse_unary()?;
            let span = start_span.merge(expr.span());
            return Ok(Expr::Borrow {
                expr: Box::new(expr),
                mutable: false,
                span,
            });
        }

        if self.match_token(&TokenKind::AmpersandMut) {
            let start_span = self.previous().span;
            let expr = self.parse_unary()?;
            let span = start_span.merge(expr.span());
            return Ok(Expr::Borrow {
                expr: Box::new(expr),
                mutable: true,
                span,
            });
        }

        self.parse_await()
    }

    /// Parse await
    fn parse_await(&mut self) -> Result<Expr, ()> {
        if self.match_token(&TokenKind::Await) {
            let start_span = self.previous().span;
            let expr = self.parse_await()?;
            let span = start_span.merge(expr.span());
            return Ok(Expr::Await {
                expr: Box::new(expr),
                span,
            });
        }

        self.parse_postfix()
    }

    /// Parse postfix (call, field access, index)
    fn parse_postfix(&mut self) -> Result<Expr, ()> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.match_token(&TokenKind::Dot) {
                let field = self.parse_field_name()?;

                // Check if it's a method call
                if self.match_token(&TokenKind::LParen) {
                    let args = self.parse_arg_list()?;
                    self.expect(&TokenKind::RParen, "expected ')' after arguments")?;

                    let span = expr.span().merge(self.previous().span);
                    expr = Expr::MethodCall {
                        object: Box::new(expr),
                        method: field,
                        args,
                        span,
                    };
                } else {
                    let span = expr.span().merge(self.previous().span);
                    expr = Expr::FieldAccess {
                        object: Box::new(expr),
                        field,
                        span,
                    };
                }
            } else if self.match_token(&TokenKind::LParen) {
                let args = self.parse_arg_list()?;
                self.expect(&TokenKind::RParen, "expected ')' after arguments")?;

                let span = expr.span().merge(self.previous().span);
                expr = Expr::Call {
                    func: Box::new(expr),
                    args,
                    span,
                };
            } else if self.match_token(&TokenKind::LBracket) {
                let index = self.parse_expression()?;
                self.expect(&TokenKind::RBracket, "expected ']' after index")?;

                let span = expr.span().merge(self.previous().span);
                expr = Expr::Index {
                    object: Box::new(expr),
                    index: Box::new(index),
                    span,
                };
            } else if self.match_token(&TokenKind::Question) {
                // Try operator: expr?
                let span = expr.span().merge(self.previous().span);
                expr = Expr::Try {
                    expr: Box::new(expr),
                    span,
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    /// Parse field/method name (allows keywords as identifiers)
    fn parse_field_name(&mut self) -> Result<String, ()> {
        let name = match &self.peek().kind {
            TokenKind::Ident(name) => name.clone(),
            // Keywords that can be used as field/method names
            TokenKind::Header => "header".to_string(),
            TokenKind::Query => "query".to_string(),
            TokenKind::Body => "body".to_string(),
            TokenKind::Transaction => "transaction".to_string(),
            TokenKind::Get => "get".to_string(),
            TokenKind::Post => "post".to_string(),
            TokenKind::Put => "put".to_string(),
            TokenKind::Delete => "delete".to_string(),
            TokenKind::Return => "return".to_string(),
            TokenKind::Async => "async".to_string(),
            TokenKind::Await => "await".to_string(),
            TokenKind::Use => "use".to_string(),
            TokenKind::Ok => "ok".to_string(),
            TokenKind::Err => "err".to_string(),
            TokenKind::Some => "some".to_string(),
            TokenKind::None => "none".to_string(),
            _ => {
                self.error_at_current("field name expected");
                return Err(());
            }
        };
        self.advance();
        Ok(name)
    }

    /// Parse primary expression
    fn parse_primary(&mut self) -> Result<Expr, ()> {
        let token = self.peek().clone();

        match &token.kind {
            TokenKind::IntLit(n) => {
                let n = *n;
                self.advance();
                Ok(Expr::IntLit(n, token.span))
            }
            TokenKind::FloatLit(n) => {
                let n = *n;
                self.advance();
                Ok(Expr::FloatLit(n, token.span))
            }
            TokenKind::StringLit(s) => {
                let s = s.clone();
                self.advance();
                Ok(Expr::StringLit(s, token.span))
            }
            TokenKind::InterpolatedString(raw_parts) => {
                let raw_parts = raw_parts.clone();
                self.advance();

                // Parse the expressions in each part
                let mut parts = Vec::new();
                for (lit, expr_str) in raw_parts {
                    // Add the literal part
                    if !lit.is_empty() {
                        parts.push(StringPart::Literal(lit));
                    }

                    // Parse the expression string if present
                    if !expr_str.is_empty() {
                        // Create a mini-lexer and parser for the expression
                        let mut expr_lexer = mendes_lexer::Lexer::new(&expr_str, 0);
                        let expr_tokens = expr_lexer.tokenize();

                        // Filter out EOF for parsing
                        let expr_tokens: Vec<_> = expr_tokens.into_iter()
                            .filter(|t| t.kind != TokenKind::Eof && t.kind != TokenKind::Newline)
                            .collect();

                        if !expr_tokens.is_empty() {
                            // Parse the expression
                            let mut expr_parser = Parser::new(expr_tokens);
                            if let Result::Ok(expr) = expr_parser.parse_expression() {
                                parts.push(StringPart::Expr(expr));
                            } else {
                                // On parse error, treat as literal
                                parts.push(StringPart::Literal(format!("{{{}}}", expr_str)));
                            }
                        }
                    }
                }

                Ok(Expr::StringInterpolation {
                    parts,
                    span: token.span,
                })
            }
            TokenKind::True => {
                self.advance();
                Ok(Expr::BoolLit(true, token.span))
            }
            TokenKind::False => {
                self.advance();
                Ok(Expr::BoolLit(false, token.span))
            }
            TokenKind::None => {
                self.advance();
                Ok(Expr::None(token.span))
            }
            TokenKind::Ok => {
                self.advance();
                self.expect(&TokenKind::LParen, "expected '(' after 'Ok'")?;
                let expr = self.parse_expression()?;
                self.expect(&TokenKind::RParen, "expected ')'")?;
                let span = token.span.merge(self.previous().span);
                Ok(Expr::Ok(Box::new(expr), span))
            }
            TokenKind::Err => {
                self.advance();
                self.expect(&TokenKind::LParen, "expected '(' after 'Err'")?;
                let expr = self.parse_expression()?;
                self.expect(&TokenKind::RParen, "expected ')'")?;
                let span = token.span.merge(self.previous().span);
                Ok(Expr::Err(Box::new(expr), span))
            }
            TokenKind::Some => {
                self.advance();
                self.expect(&TokenKind::LParen, "expected '(' after 'Some'")?;
                let expr = self.parse_expression()?;
                self.expect(&TokenKind::RParen, "expected ')'")?;
                let span = token.span.merge(self.previous().span);
                Ok(Expr::Some(Box::new(expr), span))
            }
            TokenKind::LParen => {
                self.advance();

                // Check for empty tuple
                if self.check(&TokenKind::RParen) {
                    self.advance();
                    let span = token.span.merge(self.previous().span);
                    return Ok(Expr::Tuple { elements: Vec::new(), span });
                }

                let first = self.parse_expression()?;

                // Check if it's a tuple (has comma) or just grouping
                if self.match_token(&TokenKind::Comma) {
                    let mut elements = vec![first];

                    // Parse remaining elements
                    if !self.check(&TokenKind::RParen) {
                        elements.push(self.parse_expression()?);
                        while self.match_token(&TokenKind::Comma) {
                            if self.check(&TokenKind::RParen) {
                                break; // trailing comma
                            }
                            elements.push(self.parse_expression()?);
                        }
                    }

                    self.expect(&TokenKind::RParen, "expected ')'")?;
                    let span = token.span.merge(self.previous().span);
                    Ok(Expr::Tuple { elements, span })
                } else {
                    // Just grouping: (expr)
                    self.expect(&TokenKind::RParen, "expected ')'")?;
                    Ok(first)
                }
            }
            TokenKind::LBracket => {
                self.advance();
                let mut elements = Vec::new();
                if !self.check(&TokenKind::RBracket) {
                    elements.push(self.parse_expression()?);
                    while self.match_token(&TokenKind::Comma) {
                        elements.push(self.parse_expression()?);
                    }
                }
                self.expect(&TokenKind::RBracket, "expected ']'")?;
                let span = token.span.merge(self.previous().span);
                Ok(Expr::ArrayLit(elements, span))
            }
            TokenKind::Ident(name) => {
                let name = name.clone();
                self.advance();

                // Check if it's a struct literal
                if self.check(&TokenKind::LBrace) {
                    self.advance();
                    let mut fields = Vec::new();

                    if !self.check(&TokenKind::RBrace) {
                        loop {
                            let field_name = self.parse_identifier()?;
                            self.expect(&TokenKind::Colon, "expected ':' after field name")?;
                            let value = self.parse_expression()?;
                            fields.push((field_name, value));

                            if !self.match_token(&TokenKind::Comma) {
                                break;
                            }
                        }
                    }

                    self.expect(&TokenKind::RBrace, "expected '}'")?;
                    let span = token.span.merge(self.previous().span);
                    return Ok(Expr::StructLit { name, fields, span });
                }

                Ok(Expr::Ident(name, token.span))
            }
            // `db` can be used as an expression (db.main.query...)
            TokenKind::Db => {
                self.advance();
                Ok(Expr::Ident("db".to_string(), token.span))
            }
            // `body` can be used as an expression in API handlers
            TokenKind::Body => {
                self.advance();
                Ok(Expr::Ident("body".to_string(), token.span))
            }
            // Match expression
            TokenKind::Match => {
                self.parse_match_expr()
            }
            // Closure expression: |x, y| x + y
            TokenKind::Pipe => {
                self.parse_closure()
            }
            _ => {
                self.error_at_current("expression expected");
                Err(())
            }
        }
    }

    /// Parse closure expression: |x, y| x + y or |x: int| -> int: ...
    fn parse_closure(&mut self) -> Result<Expr, ()> {
        let start_span = self.peek().span;
        self.advance(); // consume opening |

        // Parse parameters
        let mut params = Vec::new();
        if !self.check(&TokenKind::Pipe) {
            loop {
                let param_span = self.peek().span;
                let name = self.parse_identifier()?;

                // Optional type annotation
                let ty = if self.match_token(&TokenKind::Colon) {
                    Some(self.parse_type()?)
                } else {
                    None
                };

                params.push(ClosureParam {
                    name,
                    ty,
                    span: param_span.merge(self.previous().span),
                });

                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
        }

        self.expect(&TokenKind::Pipe, "expected '|' to close closure parameters")?;

        // Optional return type: -> Type
        let return_type = if self.match_token(&TokenKind::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };

        // Body: either `:` for block or expression
        let body = if self.match_token(&TokenKind::Colon) {
            // Block body
            if self.match_token(&TokenKind::Newline) {
                // Multi-line block
                let block = self.parse_block()?;
                ClosureBody::Block(block)
            } else {
                // Single expression after colon
                let expr = self.parse_expression()?;
                ClosureBody::Expr(Box::new(expr))
            }
        } else {
            // Expression body
            let expr = self.parse_expression()?;
            ClosureBody::Expr(Box::new(expr))
        };

        let span = start_span.merge(self.previous().span);

        Ok(Expr::Closure {
            params,
            return_type,
            body,
            span,
        })
    }

    // =========================================
    // Parsing helpers
    // =========================================

    fn parse_identifier(&mut self) -> Result<String, ()> {
        if let TokenKind::Ident(name) = &self.peek().kind.clone() {
            let name = name.clone();
            self.advance();
            Ok(name)
        } else {
            self.error_at_current("identifier expected");
            Err(())
        }
    }

    fn parse_type(&mut self) -> Result<Type, ()> {
        // Reference
        if self.match_token(&TokenKind::Ampersand) {
            let inner = self.parse_type()?;
            return Ok(Type::Ref(Box::new(inner)));
        }
        if self.match_token(&TokenKind::AmpersandMut) {
            let inner = self.parse_type()?;
            return Ok(Type::MutRef(Box::new(inner)));
        }

        // Array
        if self.match_token(&TokenKind::LBracket) {
            let inner = self.parse_type()?;
            self.expect(&TokenKind::RBracket, "expected ']'")?;
            return Ok(Type::Array(Box::new(inner)));
        }

        // Tuple type: (int, string, bool)
        if self.match_token(&TokenKind::LParen) {
            let mut types = Vec::new();

            if !self.check(&TokenKind::RParen) {
                types.push(self.parse_type()?);
                while self.match_token(&TokenKind::Comma) {
                    if self.check(&TokenKind::RParen) {
                        break; // trailing comma
                    }
                    types.push(self.parse_type()?);
                }
            }

            self.expect(&TokenKind::RParen, "expected ')'")?;
            return Ok(Type::Tuple(types));
        }

        // Function type: fn(int, int) -> int
        if self.match_token(&TokenKind::Fn) {
            self.expect(&TokenKind::LParen, "expected '(' for function type")?;

            let mut param_types = Vec::new();
            if !self.check(&TokenKind::RParen) {
                param_types.push(self.parse_type()?);
                while self.match_token(&TokenKind::Comma) {
                    param_types.push(self.parse_type()?);
                }
            }

            self.expect(&TokenKind::RParen, "expected ')'")?;
            self.expect(&TokenKind::Arrow, "expected '->' for function type")?;

            let return_type = self.parse_type()?;

            return Ok(Type::Function {
                params: param_types,
                return_type: Box::new(return_type),
            });
        }

        // Primitives and named types
        let name = match &self.peek().kind {
            TokenKind::IntType => {
                self.advance();
                return Ok(Type::Int);
            }
            TokenKind::FloatType => {
                self.advance();
                return Ok(Type::Float);
            }
            TokenKind::BoolType => {
                self.advance();
                return Ok(Type::Bool);
            }
            TokenKind::StringType => {
                self.advance();
                return Ok(Type::String);
            }
            TokenKind::Ident(n) => {
                let name = n.clone();
                self.advance();
                name
            }
            TokenKind::ResultType => {
                self.advance();
                "Result".to_string()
            }
            TokenKind::OptionType => {
                self.advance();
                "Option".to_string()
            }
            _ => {
                self.error_at_current("type expected");
                return Err(());
            }
        };

        // Generics
        if self.match_token(&TokenKind::Lt) {
            let mut args = Vec::new();
            args.push(self.parse_type()?);
            while self.match_token(&TokenKind::Comma) {
                args.push(self.parse_type()?);
            }
            self.expect(&TokenKind::Gt, "expected '>'")?;
            return Ok(Type::Generic { name, args });
        }

        Ok(Type::Named(name))
    }

    fn parse_param_list(&mut self) -> Result<Vec<Param>, ()> {
        let mut params = Vec::new();

        if !self.check(&TokenKind::RParen) {
            loop {
                let span = self.peek().span;
                let name = self.parse_identifier()?;
                self.expect(&TokenKind::Colon, "expected ':' after parameter name")?;
                let ty = self.parse_type()?;

                params.push(Param {
                    name,
                    ty,
                    span: span.merge(self.previous().span),
                });

                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
        }

        Ok(params)
    }

    fn parse_arg_list(&mut self) -> Result<Vec<Expr>, ()> {
        let mut args = Vec::new();

        if !self.check(&TokenKind::RParen) {
            args.push(self.parse_expression()?);
            while self.match_token(&TokenKind::Comma) {
                args.push(self.parse_expression()?);
            }
        }

        Ok(args)
    }

    fn parse_http_method(&mut self) -> Result<HttpMethod, ()> {
        let method = match &self.peek().kind {
            TokenKind::Get => HttpMethod::Get,
            TokenKind::Post => HttpMethod::Post,
            TokenKind::Put => HttpMethod::Put,
            TokenKind::Delete => HttpMethod::Delete,
            TokenKind::Patch => HttpMethod::Patch,
            _ => {
                self.error_at_current("HTTP method expected (GET, POST, PUT, DELETE, PATCH)");
                return Err(());
            }
        };
        self.advance();
        Ok(method)
    }

    fn parse_path_pattern(&mut self) -> Result<String, ()> {
        let mut path = String::new();

        // Must start with /
        if !self.match_token(&TokenKind::Slash) {
            self.error_at_current("path must start with '/'");
            return Err(());
        }
        path.push('/');

        // Parse segments
        loop {
            match &self.peek().kind {
                TokenKind::Ident(segment) => {
                    path.push_str(segment);
                    self.advance();
                }
                TokenKind::LBrace => {
                    self.advance();
                    path.push('{');

                    if let TokenKind::Ident(param) = &self.peek().kind.clone() {
                        path.push_str(&param);
                        self.advance();
                    }

                    if self.match_token(&TokenKind::Colon) {
                        path.push(':');
                        // Parameter type
                        match &self.peek().kind {
                            TokenKind::IntType => {
                                path.push_str("int");
                                self.advance();
                            }
                            TokenKind::StringType => {
                                path.push_str("string");
                                self.advance();
                            }
                            TokenKind::Ident(t) => {
                                path.push_str(t);
                                self.advance();
                            }
                            _ => {}
                        }
                    }

                    self.expect(&TokenKind::RBrace, "expected '}'")?;
                    path.push('}');
                }
                TokenKind::Slash => {
                    self.advance();
                    path.push('/');
                }
                _ => break,
            }
        }

        Ok(path)
    }

    fn parse_db_type(&mut self) -> Result<DbType, ()> {
        if let TokenKind::Ident(name) = &self.peek().kind.clone() {
            let db_type = match name.as_str() {
                "postgres" => DbType::Postgres,
                "mysql" => DbType::Mysql,
                "sqlite" => DbType::Sqlite,
                _ => {
                    self.error_at_current("database type expected (postgres, mysql, sqlite)");
                    return Err(());
                }
            };
            self.advance();
            Ok(db_type)
        } else {
            self.error_at_current("database type expected");
            Err(())
        }
    }

    fn expect_newline(&mut self) -> Result<(), ()> {
        if self.check(&TokenKind::Newline) || self.check(&TokenKind::Eof) || self.check(&TokenKind::Dedent) {
            self.match_token(&TokenKind::Newline);
            Ok(())
        } else {
            self.error_at_current("expected end of line");
            Err(())
        }
    }
}

// =========================================
// Trait to get span from Expr
// =========================================

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::IntLit(_, span) => *span,
            Expr::FloatLit(_, span) => *span,
            Expr::StringLit(_, span) => *span,
            Expr::BoolLit(_, span) => *span,
            Expr::None(span) => *span,
            Expr::Ident(_, span) => *span,
            Expr::Binary { span, .. } => *span,
            Expr::Unary { span, .. } => *span,
            Expr::Call { span, .. } => *span,
            Expr::MethodCall { span, .. } => *span,
            Expr::FieldAccess { span, .. } => *span,
            Expr::Index { span, .. } => *span,
            Expr::Await { span, .. } => *span,
            Expr::Borrow { span, .. } => *span,
            Expr::Ok(_, span) => *span,
            Expr::Err(_, span) => *span,
            Expr::Some(_, span) => *span,
            Expr::StructLit { span, .. } => *span,
            Expr::ArrayLit(_, span) => *span,
            Expr::Match { span, .. } => *span,
            Expr::Try { span, .. } => *span,
            Expr::Closure { span, .. } => *span,
            Expr::StringInterpolation { span, .. } => *span,
            Expr::Tuple { span, .. } => *span,
            Expr::Range { span, .. } => *span,
        }
    }
}

impl Pattern {
    pub fn span(&self) -> Span {
        match self {
            Pattern::Wildcard(span) => *span,
            Pattern::Literal(expr) => expr.span(),
            Pattern::Ident { span, .. } => *span,
            Pattern::Tuple(_, span) => *span,
            Pattern::Struct { span, .. } => *span,
            Pattern::Variant { span, .. } => *span,
            Pattern::Or(_, span) => *span,
            Pattern::Range { span, .. } => *span,
        }
    }
}

/// Helper function to parse
pub fn parse(tokens: Vec<Token>) -> (Program, Diagnostics) {
    let mut parser = Parser::new(tokens);
    let program = parser.parse();
    (program, parser.take_diagnostics())
}

#[cfg(test)]
mod tests {
    use super::*;
    use mendes_lexer::Lexer;

    fn parse_source(source: &str) -> (Program, Diagnostics) {
        let mut lexer = Lexer::new(source, 0);
        let tokens = lexer.tokenize();
        parse(tokens)
    }

    #[test]
    fn test_parse_let() {
        let (program, diags) = parse_source("let x: int = 10\n");
        assert!(!diags.has_errors());
        assert_eq!(program.statements.len(), 1);
        assert!(matches!(program.statements[0], Stmt::Let { .. }));
    }

    #[test]
    fn test_parse_fn() {
        let (program, diags) = parse_source("fn add(a: int, b: int) -> int:\n    return a + b\n");
        assert!(!diags.has_errors());
        assert_eq!(program.statements.len(), 1);
        assert!(matches!(program.statements[0], Stmt::Fn(_)));
    }

    #[test]
    fn test_parse_struct() {
        let (program, diags) = parse_source("struct User:\n    id: int\n    name: string\n");
        assert!(!diags.has_errors());
        assert_eq!(program.statements.len(), 1);
        assert!(matches!(program.statements[0], Stmt::Struct(_)));
    }

    #[test]
    fn test_parse_api() {
        let (program, diags) = parse_source("api GET /health:\n    return string\n\n    return \"ok\"\n");
        assert!(!diags.has_errors());
        assert_eq!(program.statements.len(), 1);
        assert!(matches!(program.statements[0], Stmt::Api(_)));
    }

    #[test]
    fn test_parse_server() {
        let (program, diags) = parse_source("server:\n    host \"0.0.0.0\"\n    port 8080\n");
        assert!(!diags.has_errors());
        assert_eq!(program.statements.len(), 1);
        assert!(matches!(program.statements[0], Stmt::Server(_)));
    }

    #[test]
    fn test_parse_from_import() {
        let (program, diags) = parse_source("from math import add, multiply\n");
        assert!(!diags.has_errors());
        assert_eq!(program.statements.len(), 1);
        match &program.statements[0] {
            Stmt::FromImport { module, items, .. } => {
                assert_eq!(module, "math");
                match items {
                    ImportItems::Names(names) => {
                        assert_eq!(names.len(), 2);
                        assert_eq!(names[0].name, "add");
                        assert_eq!(names[1].name, "multiply");
                    }
                    _ => panic!("Expected Names, got All"),
                }
            }
            _ => panic!("Expected FromImport"),
        }
    }

    #[test]
    fn test_parse_from_import_all() {
        let (program, diags) = parse_source("from utils import *\n");
        assert!(!diags.has_errors());
        assert_eq!(program.statements.len(), 1);
        match &program.statements[0] {
            Stmt::FromImport { module, items, .. } => {
                assert_eq!(module, "utils");
                assert!(matches!(items, ImportItems::All));
            }
            _ => panic!("Expected FromImport"),
        }
    }

    #[test]
    fn test_parse_from_import_with_alias() {
        let (program, diags) = parse_source("from math import add as sum, multiply as mul\n");
        assert!(!diags.has_errors());
        assert_eq!(program.statements.len(), 1);
        match &program.statements[0] {
            Stmt::FromImport { module, items, .. } => {
                assert_eq!(module, "math");
                match items {
                    ImportItems::Names(names) => {
                        assert_eq!(names.len(), 2);
                        assert_eq!(names[0].name, "add");
                        assert_eq!(names[0].alias, Some("sum".to_string()));
                        assert_eq!(names[1].name, "multiply");
                        assert_eq!(names[1].alias, Some("mul".to_string()));
                    }
                    _ => panic!("Expected Names, got All"),
                }
            }
            _ => panic!("Expected FromImport"),
        }
    }

    #[test]
    fn test_parse_module_path() {
        let (program, diags) = parse_source("from std.collections import HashMap\n");
        assert!(!diags.has_errors());
        assert_eq!(program.statements.len(), 1);
        match &program.statements[0] {
            Stmt::FromImport { module, items, .. } => {
                assert_eq!(module, "std.collections");
                match items {
                    ImportItems::Names(names) => {
                        assert_eq!(names.len(), 1);
                        assert_eq!(names[0].name, "HashMap");
                    }
                    _ => panic!("Expected Names"),
                }
            }
            _ => panic!("Expected FromImport"),
        }
    }
}
