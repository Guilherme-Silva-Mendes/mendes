//! Type Checker for the Mendes language

use crate::ownership::OwnershipChecker;
use crate::symbols::{Symbol, SymbolKind};
use crate::types::{MendesType, StructDef};
use crate::SemanticContext;
use mendes_error::{Diagnostic, Diagnostics, ErrorCode, Span};
use mendes_parser::*;

/// Main Type Checker
pub struct TypeChecker<'ctx> {
    /// Semantic context
    ctx: &'ctx mut SemanticContext,
    /// Ownership checker
    ownership: OwnershipChecker,
    /// Diagnostics
    diagnostics: Diagnostics,
    /// Return type of the current function
    current_return_type: Option<MendesType>,
    /// Whether we are in an async context
    in_async: bool,
}

impl<'ctx> TypeChecker<'ctx> {
    pub fn new(ctx: &'ctx mut SemanticContext) -> Self {
        Self {
            ctx,
            ownership: OwnershipChecker::new(),
            diagnostics: Diagnostics::new(),
            current_return_type: None,
            in_async: false,
        }
    }

    pub fn take_diagnostics(&mut self) -> Diagnostics {
        let mut diags = std::mem::take(&mut self.diagnostics);
        // Merge ownership diagnostics
        for diag in self.ownership.take_diagnostics() {
            diags.push(diag);
        }
        diags
    }

    /// Analyzes the complete program
    pub fn check_program(&mut self, program: &Program) {
        // First pass: register structs and functions
        for stmt in &program.statements {
            self.register_declarations(stmt);
        }

        // Second pass: check types
        for stmt in &program.statements {
            self.check_statement(stmt);
        }
    }

    /// First pass: register declarations
    fn register_declarations(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Struct(s) => {
                let fields: Vec<_> = s.fields.iter()
                    .map(|f| (f.name.clone(), MendesType::from_ast(&f.ty)))
                    .collect();

                // Collect methods
                let methods: Vec<_> = s.methods.iter()
                    .map(|m| {
                        let params: Vec<_> = m.params.iter()
                            .map(|p| (p.name.clone(), MendesType::from_ast(&p.ty)))
                            .collect();
                        let return_type = m.return_type.as_ref()
                            .map(MendesType::from_ast)
                            .unwrap_or(MendesType::Unit);
                        (m.name.clone(), params, return_type, m.is_async)
                    })
                    .collect();

                self.ctx.types.register_struct(StructDef {
                    name: s.name.clone(),
                    fields: fields.clone(),
                    methods: methods.clone(),
                    is_copy: s.is_copy,
                });

                self.ctx.symbols.define(Symbol {
                    name: s.name.clone(),
                    kind: SymbolKind::Struct {
                        fields,
                        methods,
                        is_copy: s.is_copy,
                    },
                    ty: MendesType::Named(s.name.clone()),
                    mutable: false,
                    defined_at: Some(s.span),
                });
            }
            Stmt::Enum(e) => {
                // Collect variants
                let variants: Vec<_> = e.variants.iter()
                    .map(|v| {
                        let data = match &v.data {
                            EnumVariantData::Unit => Vec::new(),
                            EnumVariantData::Tuple(types) => {
                                types.iter().map(MendesType::from_ast).collect()
                            }
                            EnumVariantData::Struct(fields) => {
                                fields.iter().map(|f| MendesType::from_ast(&f.ty)).collect()
                            }
                        };
                        (v.name.clone(), data)
                    })
                    .collect();

                self.ctx.symbols.define(Symbol {
                    name: e.name.clone(),
                    kind: SymbolKind::Enum {
                        variants: variants.clone(),
                    },
                    ty: MendesType::Named(e.name.clone()),
                    mutable: false,
                    defined_at: Some(e.span),
                });
            }
            Stmt::Fn(f) => {
                // Collect generic parameter names
                let generic_params: Vec<_> = f.generic_params.iter()
                    .map(|gp| gp.name.clone())
                    .collect();

                let params: Vec<_> = f.params.iter()
                    .map(|p| (p.name.clone(), MendesType::from_ast(&p.ty)))
                    .collect();
                let return_type = f.return_type.as_ref()
                    .map(MendesType::from_ast)
                    .unwrap_or(MendesType::Unit);

                self.ctx.symbols.define(Symbol {
                    name: f.name.clone(),
                    kind: SymbolKind::Function {
                        generic_params,
                        params: params.clone(),
                        return_type: return_type.clone(),
                        is_async: f.is_async,
                    },
                    ty: MendesType::Function {
                        params: params.iter().map(|(_, t)| t.clone()).collect(),
                        ret: Box::new(return_type),
                    },
                    mutable: false,
                    defined_at: Some(f.span),
                });
            }
            Stmt::Db(db) => {
                self.ctx.symbols.define(Symbol {
                    name: db.name.clone(),
                    kind: SymbolKind::Database {
                        db_type: format!("{:?}", db.db_type).to_lowercase(),
                        pool_size: db.pool_size,
                    },
                    ty: MendesType::Named(format!("Database<{:?}>", db.db_type)),
                    mutable: false,
                    defined_at: Some(db.span),
                });
            }
            Stmt::Middleware(m) => {
                self.ctx.symbols.define(Symbol {
                    name: m.name.clone(),
                    kind: SymbolKind::Middleware,
                    ty: MendesType::Function {
                        params: vec![],
                        ret: Box::new(MendesType::Unit),
                    },
                    mutable: false,
                    defined_at: Some(m.span),
                });
            }
            _ => {}
        }
    }

    /// Checks a statement
    fn check_statement(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Import { .. } => {
                // Imports are processed during the module loading phase
                // Here we just ignore them - the CLI is responsible for resolving imports
            }
            Stmt::FromImport { .. } => {
                // From imports are processed during the module loading phase
                // The CLI is responsible for resolving imports and merging symbols
            }
            Stmt::Let { name, ty, value, mutable, span } => {
                self.check_let(name, ty.as_ref(), value, *mutable, *span);
            }
            Stmt::Fn(f) => {
                self.check_fn(f);
            }
            Stmt::Struct(s) => {
                self.check_struct(s);
            }
            Stmt::Enum(e) => {
                self.check_enum(e);
            }
            Stmt::Api(api) => {
                self.check_api(api);
            }
            Stmt::WebSocket(ws) => {
                self.check_websocket(ws);
            }
            Stmt::Server(_) => {
                // Server does not need type checking
            }
            Stmt::Middleware(m) => {
                self.check_middleware(m);
            }
            Stmt::Db(_) => {
                // DB declaration has already been registered
            }
            Stmt::If { condition, then_block, else_block, span } => {
                self.check_if(condition, then_block, else_block.as_deref(), *span);
            }
            Stmt::For { var, iter, body, span } => {
                self.check_for(var, iter, body, *span);
            }
            Stmt::While { condition, body, span } => {
                self.check_while(condition, body, *span);
            }
            Stmt::Return { value, span } => {
                self.check_return(value.as_ref(), *span);
            }
            Stmt::Expr(expr) => {
                self.check_expr(expr);
            }
            Stmt::Trait(t) => {
                self.check_trait(t);
            }
            Stmt::ImplTrait(i) => {
                self.check_impl_trait(i);
            }
            Stmt::TypeAlias { name, ty, span } => {
                self.check_type_alias(name, ty, *span);
            }
            Stmt::Break { span } => {
                // Break is valid inside loops - no type checking needed
                // Loop context checking would be done here if implemented
                let _ = span;
            }
            Stmt::Continue { span } => {
                // Continue is valid inside loops - no type checking needed
                let _ = span;
            }
        }
    }

    /// Checks trait declaration
    fn check_trait(&mut self, t: &TraitDecl) {
        // Register the trait in the symbol table
        self.ctx.symbols.define(Symbol::new(
            t.name.clone(),
            MendesType::Named(t.name.clone()),
            SymbolKind::Trait,
            t.span,
        ));

        // Check method signatures
        for method in &t.methods {
            // Validate parameter types
            for param in &method.params {
                let _ty = MendesType::from_ast(&param.ty);
            }
            // Validate return type
            if let Some(ret) = &method.return_type {
                let _ty = MendesType::from_ast(ret);
            }
        }
    }

    /// Checks trait implementation
    fn check_impl_trait(&mut self, i: &ImplTraitDecl) {
        // Check that the type exists
        if self.ctx.symbols.lookup(&i.type_name).is_none() {
            self.diagnostics.push(
                Diagnostic::error(format!("type `{}` not found", i.type_name))
                    .with_code(ErrorCode::UNKNOWN_TYPE)
                    .with_label(i.span, "implementing trait for unknown type")
            );
        }

        // Check that the trait exists
        if self.ctx.symbols.lookup(&i.trait_name).is_none() {
            self.diagnostics.push(
                Diagnostic::error(format!("trait `{}` not found", i.trait_name))
                    .with_code(ErrorCode::UNKNOWN_TYPE)
                    .with_label(i.span, "unknown trait")
            );
        }

        // Check each method implementation
        for method in &i.methods {
            self.ctx.symbols.push_scope();
            self.ownership.push_scope();

            // Register self parameter based on receiver
            let self_type = MendesType::Named(i.type_name.clone());
            match method.receiver {
                MethodReceiver::Ref => {
                    self.ctx.symbols.define(Symbol::variable(
                        "self".to_string(),
                        MendesType::Ref(Box::new(self_type)),
                        false,
                        method.span,
                    ));
                }
                MethodReceiver::MutRef => {
                    self.ctx.symbols.define(Symbol::variable(
                        "self".to_string(),
                        MendesType::MutRef(Box::new(self_type)),
                        false,
                        method.span,
                    ));
                }
                MethodReceiver::Value => {
                    self.ctx.symbols.define(Symbol::variable(
                        "self".to_string(),
                        self_type,
                        false,
                        method.span,
                    ));
                }
            }

            // Register parameters
            for param in &method.params {
                let ty = MendesType::from_ast(&param.ty);
                self.ctx.symbols.define(Symbol::parameter(
                    param.name.clone(),
                    ty.clone(),
                    param.span,
                ));
            }

            // Check method body
            for stmt in &method.body {
                self.check_statement(stmt);
            }

            self.ownership.pop_scope();
            self.ctx.symbols.pop_scope();
        }
    }

    /// Checks type alias
    fn check_type_alias(&mut self, name: &str, ty: &Type, span: Span) {
        let mendes_type = MendesType::from_ast(ty);

        // Register the type alias
        self.ctx.symbols.define(Symbol::new(
            name.to_string(),
            mendes_type,
            SymbolKind::TypeAlias,
            span,
        ));
    }

    /// Checks let declaration
    fn check_let(&mut self, name: &str, ty: Option<&Type>, value: &Expr, mutable: bool, span: Span) {
        let value_type = self.check_expr(value);

        let declared_type = ty.map(MendesType::from_ast);

        let final_type = if let Some(declared) = &declared_type {
            if !declared.is_compatible_with(&value_type) {
                self.diagnostics.push(
                    Diagnostic::error(format!(
                        "incompatible type: expected `{}`, found `{}`",
                        declared, value_type
                    ))
                    .with_code(ErrorCode::TYPE_MISMATCH)
                    .with_label(span, "incompatible types here")
                );
            }
            declared.clone()
        } else {
            value_type.clone()
        };

        // Register in the symbol table
        self.ctx.symbols.define(Symbol::variable(
            name.to_string(),
            final_type.clone(),
            mutable,
            span,
        ));

        // Register in the ownership checker
        self.ownership.define(name.to_string(), final_type, mutable, span);
    }

    /// Checks function
    fn check_fn(&mut self, f: &FnDecl) {
        self.ctx.symbols.push_scope();
        self.ownership.push_scope();

        // Register generic type parameters
        for gp in &f.generic_params {
            self.ctx.types.register_generic_param(&gp.name);
        }

        // Register parameters
        for param in &f.params {
            let ty = MendesType::from_ast(&param.ty);
            self.ctx.symbols.define(Symbol::parameter(
                param.name.clone(),
                ty.clone(),
                param.span,
            ));
            self.ownership.define(param.name.clone(), ty, false, param.span);
        }

        // Define the expected return type
        let return_type = f.return_type.as_ref()
            .map(MendesType::from_ast)
            .unwrap_or(MendesType::Unit);
        self.current_return_type = Some(return_type);

        // Async context
        if f.is_async {
            self.in_async = true;
            self.ownership.enter_async();
        }

        // Check body
        for stmt in &f.body {
            self.check_statement(stmt);
        }

        if f.is_async {
            self.in_async = false;
            self.ownership.exit_async();
        }

        self.current_return_type = None;

        // Unregister generic type parameters
        for gp in &f.generic_params {
            self.ctx.types.unregister_generic_param(&gp.name);
        }

        self.ownership.pop_scope();
        self.ctx.symbols.pop_scope();
    }

    /// Checks struct
    fn check_struct(&mut self, s: &StructDecl) {
        // Register generic type parameters temporarily
        for gp in &s.generic_params {
            self.ctx.types.register_generic_param(&gp.name);
        }

        for field in &s.fields {
            let ty = MendesType::from_ast(&field.ty);
            if !self.ctx.types.type_exists(&ty) {
                self.diagnostics.push(
                    Diagnostic::error(format!("unknown type: `{}`", ty))
                        .with_code(ErrorCode::UNKNOWN_TYPE)
                        .with_label(field.span, "type not found")
                );
            }
        }

        // Unregister generic type parameters
        for gp in &s.generic_params {
            self.ctx.types.unregister_generic_param(&gp.name);
        }
    }

    /// Checks enum
    fn check_enum(&mut self, e: &EnumDecl) {
        // Check that all variant types exist
        for variant in &e.variants {
            match &variant.data {
                EnumVariantData::Unit => {
                    // No types to check
                }
                EnumVariantData::Tuple(types) => {
                    for ty in types {
                        let mendes_ty = MendesType::from_ast(ty);
                        if !self.ctx.types.type_exists(&mendes_ty) {
                            self.diagnostics.push(
                                Diagnostic::error(format!("unknown type: `{}`", mendes_ty))
                                    .with_code(ErrorCode::UNKNOWN_TYPE)
                                    .with_label(variant.span, "type not found")
                            );
                        }
                    }
                }
                EnumVariantData::Struct(fields) => {
                    for field in fields {
                        let mendes_ty = MendesType::from_ast(&field.ty);
                        if !self.ctx.types.type_exists(&mendes_ty) {
                            self.diagnostics.push(
                                Diagnostic::error(format!("unknown type: `{}`", mendes_ty))
                                    .with_code(ErrorCode::UNKNOWN_TYPE)
                                    .with_label(field.span, "type not found")
                            );
                        }
                    }
                }
            }
        }
    }

    /// Checks API
    fn check_api(&mut self, api: &ApiDecl) {
        self.ctx.symbols.push_scope();
        self.ownership.push_scope();

        // Register `body` if declared
        if let Some(body_type) = &api.body_type {
            let ty = MendesType::from_ast(body_type);
            self.ctx.symbols.define(Symbol::parameter(
                "body".to_string(),
                ty.clone(),
                api.span,
            ));
            self.ownership.define("body".to_string(), ty, false, api.span);
        }

        // Register path parameters
        // E.g.: /users/{id:int} -> registers `id` as int
        self.register_path_params(&api.path, api.span);

        // Define return type
        let return_type = api.return_type.as_ref()
            .map(MendesType::from_ast)
            .unwrap_or(MendesType::Unit);
        self.current_return_type = Some(return_type);

        // Async context
        if api.is_async {
            self.in_async = true;
            self.ownership.enter_async();
        }

        // Check handler
        for stmt in &api.handler {
            self.check_statement(stmt);
        }

        if api.is_async {
            self.in_async = false;
            self.ownership.exit_async();
        }

        self.current_return_type = None;
        self.ownership.pop_scope();
        self.ctx.symbols.pop_scope();
    }

    /// Checks WebSocket declaration
    fn check_websocket(&mut self, ws: &WsDecl) {
        // Register path parameters
        self.ctx.symbols.push_scope();
        self.ownership.push_scope();

        self.register_path_params(&ws.path, ws.span);

        // Register special WebSocket variables
        // `conn` - the WebSocket connection
        self.ctx.symbols.define(Symbol::variable(
            "conn".to_string(),
            MendesType::Named("WsConnection".to_string()),
            false,
            ws.span,
        ));

        // Check on_connect handler
        if let Some(handler) = &ws.on_connect {
            self.ctx.symbols.push_scope();
            for stmt in handler {
                self.check_statement(stmt);
            }
            self.ctx.symbols.pop_scope();
        }

        // Check on_message handler
        if let Some(handler) = &ws.on_message {
            self.ctx.symbols.push_scope();
            // Register `message` variable
            self.ctx.symbols.define(Symbol::variable(
                "message".to_string(),
                MendesType::String,
                false,
                ws.span,
            ));
            for stmt in handler {
                self.check_statement(stmt);
            }
            self.ctx.symbols.pop_scope();
        }

        // Check on_disconnect handler
        if let Some(handler) = &ws.on_disconnect {
            self.ctx.symbols.push_scope();
            for stmt in handler {
                self.check_statement(stmt);
            }
            self.ctx.symbols.pop_scope();
        }

        self.ownership.pop_scope();
        self.ctx.symbols.pop_scope();
    }

    /// Registers path parameters
    fn register_path_params(&mut self, path: &str, span: Span) {
        // Simple parsing of {param:type}
        let mut chars = path.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '{' {
                let mut param = String::new();
                let mut ty_str = String::new();
                let mut in_type = false;

                while let Some(&c) = chars.peek() {
                    chars.next();
                    if c == '}' {
                        break;
                    } else if c == ':' {
                        in_type = true;
                    } else if in_type {
                        ty_str.push(c);
                    } else {
                        param.push(c);
                    }
                }

                if !param.is_empty() {
                    let ty = match ty_str.as_str() {
                        "int" => MendesType::Int,
                        "string" => MendesType::String,
                        _ => MendesType::String,
                    };

                    self.ctx.symbols.define(Symbol::parameter(param.clone(), ty.clone(), span));
                    self.ownership.define(param, ty, false, span);
                }
            }
        }
    }

    /// Checks middleware
    fn check_middleware(&mut self, m: &MiddlewareDecl) {
        self.ctx.symbols.push_scope();
        self.ownership.push_scope();

        // `request` is available in middlewares
        self.ctx.symbols.define(Symbol::parameter(
            "request".to_string(),
            MendesType::Named("Request".to_string()),
            m.span,
        ));

        for stmt in &m.body {
            self.check_statement(stmt);
        }

        self.ownership.pop_scope();
        self.ctx.symbols.pop_scope();
    }

    /// Checks if
    fn check_if(&mut self, condition: &Expr, then_block: &[Stmt], else_block: Option<&[Stmt]>, _span: Span) {
        let cond_type = self.check_expr(condition);
        if !cond_type.is_compatible_with(&MendesType::Bool) {
            self.diagnostics.push(
                Diagnostic::error(format!("condition must be `bool`, found `{}`", cond_type))
                    .with_code(ErrorCode::TYPE_MISMATCH)
                    .with_label(condition.span(), "expected bool")
            );
        }

        self.ctx.symbols.push_scope();
        self.ownership.push_scope();
        for stmt in then_block {
            self.check_statement(stmt);
        }
        self.ownership.pop_scope();
        self.ctx.symbols.pop_scope();

        if let Some(else_stmts) = else_block {
            self.ctx.symbols.push_scope();
            self.ownership.push_scope();
            for stmt in else_stmts {
                self.check_statement(stmt);
            }
            self.ownership.pop_scope();
            self.ctx.symbols.pop_scope();
        }
    }

    /// Checks for
    fn check_for(&mut self, var: &str, iter: &Expr, body: &[Stmt], span: Span) {
        let iter_type = self.check_expr(iter);

        // Determine element type
        let elem_type = match &iter_type {
            MendesType::Array(inner) => (**inner).clone(),
            MendesType::Range(inner) => (**inner).clone(),
            MendesType::String => MendesType::String, // Iterate over chars as strings
            _ => {
                self.diagnostics.push(
                    Diagnostic::error(format!("expected iterable type, found `{}`", iter_type))
                        .with_code(ErrorCode::TYPE_MISMATCH)
                        .with_label(iter.span(), "not iterable")
                );
                MendesType::Unknown
            }
        };

        self.ctx.symbols.push_scope();
        self.ownership.push_scope();

        self.ctx.symbols.define(Symbol::variable(var.to_string(), elem_type.clone(), false, span));
        self.ownership.define(var.to_string(), elem_type, false, span);

        for stmt in body {
            self.check_statement(stmt);
        }

        self.ownership.pop_scope();
        self.ctx.symbols.pop_scope();
    }

    /// Checks while
    fn check_while(&mut self, condition: &Expr, body: &[Stmt], _span: Span) {
        let cond_type = self.check_expr(condition);
        if !cond_type.is_compatible_with(&MendesType::Bool) {
            self.diagnostics.push(
                Diagnostic::error(format!("condition must be `bool`, found `{}`", cond_type))
                    .with_code(ErrorCode::TYPE_MISMATCH)
                    .with_label(condition.span(), "expected bool")
            );
        }

        self.ctx.symbols.push_scope();
        self.ownership.push_scope();
        for stmt in body {
            self.check_statement(stmt);
        }
        self.ownership.pop_scope();
        self.ctx.symbols.pop_scope();
    }

    /// Checks return
    fn check_return(&mut self, value: Option<&Expr>, span: Span) {
        let return_type = value.map(|e| self.check_expr(e)).unwrap_or(MendesType::Unit);

        if let Some(expected) = &self.current_return_type {
            if !expected.is_compatible_with(&return_type) {
                self.diagnostics.push(
                    Diagnostic::error(format!(
                        "incompatible return type: expected `{}`, found `{}`",
                        expected, return_type
                    ))
                    .with_code(ErrorCode::TYPE_MISMATCH)
                    .with_label(span, "incompatible type")
                );
            }
        }
    }

    /// Checks expression and returns its type
    fn check_expr(&mut self, expr: &Expr) -> MendesType {
        match expr {
            Expr::IntLit(_, _) => MendesType::Int,
            Expr::FloatLit(_, _) => MendesType::Float,
            Expr::StringLit(_, _) => MendesType::String,
            Expr::BoolLit(_, _) => MendesType::Bool,
            Expr::None(_) => MendesType::Named("None".to_string()),

            Expr::Ident(name, span) => {
                // Check usage in ownership
                if let Err(diag) = self.ownership.check_use(name, *span) {
                    self.diagnostics.push(diag);
                }

                if let Some(symbol) = self.ctx.symbols.lookup(name) {
                    symbol.ty.clone()
                } else {
                    self.diagnostics.push(
                        Diagnostic::error(format!("variable not found: `{}`", name))
                            .with_code(ErrorCode::UNKNOWN_VARIABLE)
                            .with_label(*span, "not declared in this scope")
                    );
                    MendesType::Unknown
                }
            }

            Expr::Binary { left, op, right, span } => {
                self.check_binary(left, *op, right, *span)
            }

            Expr::Unary { op, expr, span } => {
                self.check_unary(*op, expr, *span)
            }

            Expr::Call { func, args, span } => {
                self.check_call(func, args, *span)
            }

            Expr::MethodCall { object, method, args, span } => {
                self.check_method_call(object, method, args, *span)
            }

            Expr::FieldAccess { object, field, span } => {
                self.check_field_access(object, field, *span)
            }

            Expr::Index { object, index, span } => {
                self.check_index(object, index, *span)
            }

            Expr::Await { expr, span } => {
                if !self.in_async {
                    self.diagnostics.push(
                        Diagnostic::error("await can only be used in async context")
                            .with_code(ErrorCode::INVALID_SYNTAX)
                            .with_label(*span, "await outside of async function")
                    );
                }

                // Check borrows crossing await
                self.ownership.check_await(*span);

                let inner_type = self.check_expr(expr);
                match inner_type {
                    MendesType::Future(inner) => *inner,
                    _ => inner_type, // Allows await on any expression for simplification
                }
            }

            Expr::Borrow { expr, mutable, span } => {
                let inner_type = self.check_expr(expr);

                // Register borrow in ownership checker
                if let Expr::Ident(name, _) = expr.as_ref() {
                    let result = if *mutable {
                        self.ownership.borrow_mut(name, *span)
                    } else {
                        self.ownership.borrow(name, *span)
                    };

                    if let Err(diag) = result {
                        self.diagnostics.push(diag);
                    }
                }

                if *mutable {
                    MendesType::MutRef(Box::new(inner_type))
                } else {
                    MendesType::Ref(Box::new(inner_type))
                }
            }

            Expr::Ok(inner, _) => {
                let inner_type = self.check_expr(inner);
                MendesType::Generic {
                    name: "Result".to_string(),
                    args: vec![inner_type, MendesType::Unknown],
                }
            }

            Expr::Err(inner, _) => {
                let inner_type = self.check_expr(inner);
                MendesType::Generic {
                    name: "Result".to_string(),
                    args: vec![MendesType::Unknown, inner_type],
                }
            }

            Expr::Some(inner, _) => {
                let inner_type = self.check_expr(inner);
                MendesType::Generic {
                    name: "Option".to_string(),
                    args: vec![inner_type],
                }
            }

            Expr::StructLit { name, fields, span } => {
                self.check_struct_lit(name, fields, *span)
            }

            Expr::ArrayLit(elements, _) => {
                if elements.is_empty() {
                    MendesType::Array(Box::new(MendesType::Unknown))
                } else {
                    let first_type = self.check_expr(&elements[0]);
                    for elem in &elements[1..] {
                        let elem_type = self.check_expr(elem);
                        if !first_type.is_compatible_with(&elem_type) {
                            self.diagnostics.push(
                                Diagnostic::error("array elements have different types")
                                    .with_code(ErrorCode::TYPE_MISMATCH)
                                    .with_label(elem.span(), format!("expected `{}`, found `{}`", first_type, elem_type))
                            );
                        }
                    }
                    MendesType::Array(Box::new(first_type))
                }
            }

            Expr::Match { expr, arms, span } => {
                self.check_match(expr, arms, *span)
            }

            Expr::Try { expr, span } => {
                self.check_try(expr, *span)
            }

            Expr::Closure { params, return_type, body, span } => {
                self.check_closure(params, return_type, body, *span)
            }

            Expr::StringInterpolation { parts, span: _ } => {
                // Check all expressions in the interpolation
                for part in parts {
                    if let StringPart::Expr(expr) = part {
                        // The expression should be formattable (any type works with string conversion)
                        self.check_expr(expr);
                    }
                }
                MendesType::String
            }

            Expr::Tuple { elements, span: _ } => {
                let element_types: Vec<MendesType> = elements.iter().map(|e| self.check_expr(e)).collect();
                MendesType::Tuple(element_types)
            }

            Expr::Range { start, end, inclusive: _, span: _ } => {
                // Determine the element type from start or end
                let element_type = if let Some(s) = start {
                    self.check_expr(s)
                } else if let Some(e) = end {
                    self.check_expr(e)
                } else {
                    MendesType::Int // Default to int for unbounded ranges
                };

                // Check that start and end have compatible types
                if let (Some(s), Some(e)) = (start, end) {
                    let start_type = self.check_expr(s);
                    let end_type = self.check_expr(e);
                    if !start_type.is_compatible_with(&end_type) {
                        self.diagnostics.push(
                            Diagnostic::error(format!(
                                "range bounds must have compatible types: found `{}` and `{}`",
                                start_type, end_type
                            ))
                            .with_code(ErrorCode::TYPE_MISMATCH)
                            .with_label(s.span().merge(e.span()), "incompatible range bounds")
                        );
                    }
                }

                MendesType::Range(Box::new(element_type))
            }
        }
    }

    /// Checks closure expression
    fn check_closure(&mut self, params: &[ClosureParam], return_type: &Option<Type>, body: &ClosureBody, _span: Span) -> MendesType {
        // Enter new scope for closure body
        self.ctx.symbols.push_scope();
        self.ownership.push_scope();

        // Add parameters to scope
        let param_types: Vec<MendesType> = params.iter().map(|p| {
            let ty = p.ty.as_ref()
                .map(MendesType::from_ast)
                .unwrap_or(MendesType::Unknown);

            self.ctx.symbols.define(Symbol::variable(
                p.name.clone(),
                ty.clone(),
                false,
                p.span,
            ));
            self.ownership.define(p.name.clone(), ty.clone(), false, p.span);
            ty
        }).collect();

        // Check body
        let body_type = match body {
            ClosureBody::Expr(expr) => {
                self.check_expr(expr)
            }
            ClosureBody::Block(stmts) => {
                let mut result_type = MendesType::Unit;
                for stmt in stmts {
                    if let Stmt::Return { value, .. } = stmt {
                        if let Some(val) = value {
                            result_type = self.check_expr(val);
                        }
                    } else {
                        self.check_statement(stmt);
                    }
                }
                result_type
            }
        };

        // Pop closure scope
        self.ownership.pop_scope();
        self.ctx.symbols.pop_scope();

        // Determine return type
        let ret_type = return_type.as_ref()
            .map(MendesType::from_ast)
            .unwrap_or(body_type);

        // Return a function type
        MendesType::Generic {
            name: "Fn".to_string(),
            args: {
                let mut args = param_types;
                args.push(ret_type);
                args
            }
        }
    }

    /// Checks try expression (?)
    fn check_try(&mut self, expr: &Expr, span: Span) -> MendesType {
        let expr_type = self.check_expr(expr);

        // The ? operator works on Result<T, E> and Option<T>
        match &expr_type {
            MendesType::Generic { name, args } if name == "Result" => {
                // Result<T, E>? returns T, propagates E
                args.first().cloned().unwrap_or(MendesType::Unknown)
            }
            MendesType::Generic { name, args } if name == "Option" => {
                // Option<T>? returns T, propagates None
                args.first().cloned().unwrap_or(MendesType::Unknown)
            }
            MendesType::Unknown => MendesType::Unknown,
            _ => {
                self.diagnostics.push(
                    Diagnostic::error(format!(
                        "the `?` operator can only be applied to `Result` or `Option`, found `{}`",
                        expr_type
                    ))
                    .with_code(ErrorCode::TYPE_MISMATCH)
                    .with_label(span, "cannot use `?` here")
                );
                MendesType::Unknown
            }
        }
    }

    /// Checks match expression
    fn check_match(&mut self, expr: &Expr, arms: &[MatchArm], span: Span) -> MendesType {
        let scrutinee_type = self.check_expr(expr);

        // Track arm return types for unification
        let mut arm_types: Vec<MendesType> = Vec::new();

        for arm in arms {
            self.ctx.symbols.push_scope();
            self.ownership.push_scope();

            // Check pattern and bind variables
            self.check_pattern(&arm.pattern, &scrutinee_type);

            // Check guard if present
            if let Some(guard) = &arm.guard {
                let guard_type = self.check_expr(guard);
                if !guard_type.is_compatible_with(&MendesType::Bool) {
                    self.diagnostics.push(
                        Diagnostic::error(format!("match guard must be `bool`, found `{}`", guard_type))
                            .with_code(ErrorCode::TYPE_MISMATCH)
                            .with_label(guard.span(), "expected bool")
                    );
                }
            }

            // Check arm body
            let mut arm_type = MendesType::Unit;
            for stmt in &arm.body {
                match stmt {
                    Stmt::Return { value, .. } => {
                        if let Some(val) = value.as_ref() {
                            arm_type = self.check_expr(val);
                        }
                    }
                    Stmt::Expr(Expr::Ok(val, _)) | Stmt::Expr(Expr::Err(val, _)) => {
                        arm_type = self.check_expr(val);
                    }
                    Stmt::Expr(e) => {
                        arm_type = self.check_expr(e);
                    }
                    _ => {
                        self.check_statement(stmt);
                    }
                }
            }

            arm_types.push(arm_type);

            self.ownership.pop_scope();
            self.ctx.symbols.pop_scope();
        }

        // All arms should have compatible types
        if let Some(first_type) = arm_types.first() {
            for (i, arm_type) in arm_types.iter().enumerate().skip(1) {
                if !first_type.is_compatible_with(arm_type) {
                    self.diagnostics.push(
                        Diagnostic::error(format!(
                            "match arms have incompatible types: expected `{}`, found `{}`",
                            first_type, arm_type
                        ))
                        .with_code(ErrorCode::TYPE_MISMATCH)
                        .with_label(arms[i].span, "incompatible type in this arm")
                    );
                }
            }
            first_type.clone()
        } else {
            self.diagnostics.push(
                Diagnostic::error("match expression must have at least one arm")
                    .with_code(ErrorCode::INVALID_SYNTAX)
                    .with_label(span, "empty match")
            );
            MendesType::Unknown
        }
    }

    /// Checks a pattern and binds variables
    fn check_pattern(&mut self, pattern: &Pattern, expected_type: &MendesType) {
        match pattern {
            Pattern::Wildcard(_) => {
                // Wildcard matches anything, no bindings
            }

            Pattern::Literal(expr) => {
                let lit_type = self.check_expr(expr);
                if !expected_type.is_compatible_with(&lit_type) {
                    self.diagnostics.push(
                        Diagnostic::error(format!(
                            "pattern type `{}` does not match expected `{}`",
                            lit_type, expected_type
                        ))
                        .with_code(ErrorCode::TYPE_MISMATCH)
                        .with_label(expr.span(), "incompatible pattern")
                    );
                }
            }

            Pattern::Ident { name, mutable, span } => {
                // Bind the variable
                self.ctx.symbols.define(Symbol::variable(
                    name.clone(),
                    expected_type.clone(),
                    *mutable,
                    *span,
                ));
                self.ownership.define(name.clone(), expected_type.clone(), *mutable, *span);
            }

            Pattern::Tuple(patterns, span) => {
                // For now, just bind each pattern as Unknown
                // Full tuple type support would require tuple types in MendesType
                for pat in patterns {
                    self.check_pattern(pat, &MendesType::Unknown);
                }
                let _ = span; // suppress unused warning
            }

            Pattern::Struct { name, fields, span } => {
                // Check that the struct exists and fields match
                if let Some(struct_def) = self.ctx.types.get_struct(name).cloned() {
                    for (field_name, field_pattern) in fields {
                        let field_type = struct_def.fields.iter()
                            .find(|(n, _)| n == field_name)
                            .map(|(_, t)| t.clone())
                            .unwrap_or_else(|| {
                                self.diagnostics.push(
                                    Diagnostic::error(format!("field `{}` not found in struct `{}`", field_name, name))
                                        .with_code(ErrorCode::UNKNOWN_VARIABLE)
                                        .with_label(*span, "unknown field")
                                );
                                MendesType::Unknown
                            });

                        if let Some(pat) = field_pattern {
                            self.check_pattern(pat, &field_type);
                        } else {
                            // Shorthand { x } means bind x to field x
                            self.ctx.symbols.define(Symbol::variable(
                                field_name.clone(),
                                field_type.clone(),
                                false,
                                *span,
                            ));
                            self.ownership.define(field_name.clone(), field_type, false, *span);
                        }
                    }
                }
            }

            Pattern::Variant { enum_name, variant, data, span } => {
                // Handle built-in types like Option and Result
                let is_option_none = enum_name.as_ref().map(|n| n == "Option").unwrap_or(false) && variant == "None";
                let is_option_some = enum_name.as_ref().map(|n| n == "Option").unwrap_or(false) && variant == "Some";
                let is_result_ok = enum_name.as_ref().map(|n| n == "Result").unwrap_or(false) && variant == "Ok";
                let is_result_err = enum_name.as_ref().map(|n| n == "Result").unwrap_or(false) && variant == "Err";

                if is_option_none {
                    // None matches Option<T>
                } else if is_option_some || is_result_ok || is_result_err {
                    // Extract inner type from generic
                    let inner_type = match expected_type {
                        MendesType::Generic { name: _, args } => {
                            if is_result_err {
                                args.get(1).cloned().unwrap_or(MendesType::Unknown)
                            } else {
                                args.first().cloned().unwrap_or(MendesType::Unknown)
                            }
                        }
                        _ => MendesType::Unknown,
                    };

                    if let VariantPatternData::Tuple(patterns) = data {
                        for pat in patterns {
                            self.check_pattern(pat, &inner_type);
                        }
                    }
                } else if let Some(enum_name_str) = enum_name {
                    // User-defined enum
                    // Clone to avoid borrow conflict
                    let variant_types_opt = self.ctx.symbols.lookup(enum_name_str)
                        .and_then(|symbol| {
                            if let SymbolKind::Enum { variants } = &symbol.kind {
                                variants.iter().find(|(n, _)| n == variant)
                                    .map(|(_, types)| types.clone())
                            } else {
                                None
                            }
                        });

                    if let Some(variant_types) = variant_types_opt {
                        match data {
                            VariantPatternData::Unit => {
                                // No bindings needed
                            }
                            VariantPatternData::Tuple(patterns) => {
                                for (pat, ty) in patterns.iter().zip(variant_types.iter()) {
                                    self.check_pattern(pat, ty);
                                }
                            }
                            VariantPatternData::Struct(fields) => {
                                // For struct variants, we'd need field names stored
                                for (field_name, field_pattern) in fields {
                                    if let Some(pat) = field_pattern {
                                        self.check_pattern(pat, &MendesType::Unknown);
                                    } else {
                                        self.ctx.symbols.define(Symbol::variable(
                                            field_name.clone(),
                                            MendesType::Unknown,
                                            false,
                                            *span,
                                        ));
                                    }
                                }
                            }
                        }
                    } else {
                        // Check if enum exists but variant doesn't
                        let enum_exists = self.ctx.symbols.lookup(enum_name_str).is_some();
                        if enum_exists {
                            self.diagnostics.push(
                                Diagnostic::error(format!("variant `{}` not found in enum `{}`", variant, enum_name_str))
                                    .with_code(ErrorCode::UNKNOWN_VARIABLE)
                                    .with_label(*span, "unknown variant")
                            );
                        }
                    }
                } else {
                    // Unqualified variant - try to infer from expected type
                    if let MendesType::Named(type_name) = expected_type {
                        // Clone to avoid borrow conflict
                        let variant_types_opt = self.ctx.symbols.lookup(type_name)
                            .and_then(|symbol| {
                                if let SymbolKind::Enum { variants } = &symbol.kind {
                                    variants.iter().find(|(n, _)| n == variant)
                                        .map(|(_, types)| types.clone())
                                } else {
                                    None
                                }
                            });

                        if let Some(variant_types) = variant_types_opt {
                            if let VariantPatternData::Tuple(patterns) = data {
                                for (pat, ty) in patterns.iter().zip(variant_types.iter()) {
                                    self.check_pattern(pat, ty);
                                }
                            }
                        }
                    }
                }
            }

            Pattern::Or(patterns, _span) => {
                // All patterns in an or-pattern must match the same type
                for pat in patterns {
                    self.check_pattern(pat, expected_type);
                }
            }

            Pattern::Range { start, end, inclusive: _, span } => {
                // Check that range bounds are compatible with expected type
                if let Some(s) = start {
                    let start_type = self.check_expr(s);
                    if !expected_type.is_compatible_with(&start_type) {
                        self.diagnostics.push(
                            Diagnostic::error(format!(
                                "range start type `{}` does not match expected `{}`",
                                start_type, expected_type
                            ))
                            .with_code(ErrorCode::TYPE_MISMATCH)
                            .with_label(*span, "incompatible range")
                        );
                    }
                }
                if let Some(e) = end {
                    let end_type = self.check_expr(e);
                    if !expected_type.is_compatible_with(&end_type) {
                        self.diagnostics.push(
                            Diagnostic::error(format!(
                                "range end type `{}` does not match expected `{}`",
                                end_type, expected_type
                            ))
                            .with_code(ErrorCode::TYPE_MISMATCH)
                            .with_label(*span, "incompatible range")
                        );
                    }
                }
            }
        }
    }

    /// Checks binary operation
    fn check_binary(&mut self, left: &Expr, op: BinOp, right: &Expr, span: Span) -> MendesType {
        let left_type = self.check_expr(left);
        let right_type = self.check_expr(right);

        match op {
            // Arithmetic
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => {
                // String + String is concatenation
                if matches!(op, BinOp::Add) && matches!((&left_type, &right_type), (MendesType::String, MendesType::String)) {
                    return MendesType::String;
                }

                if !matches!((&left_type, &right_type),
                    (MendesType::Int, MendesType::Int) |
                    (MendesType::Float, MendesType::Float) |
                    (MendesType::Unknown, _) | (_, MendesType::Unknown)
                ) {
                    self.diagnostics.push(
                        Diagnostic::error(format!("operation `{:?}` not supported between `{}` and `{}`", op, left_type, right_type))
                            .with_code(ErrorCode::TYPE_MISMATCH)
                            .with_label(span, "incompatible types")
                    );
                }

                if matches!(left_type, MendesType::Float) || matches!(right_type, MendesType::Float) {
                    MendesType::Float
                } else {
                    MendesType::Int
                }
            }

            // Comparison
            BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
                if !left_type.is_compatible_with(&right_type) {
                    self.diagnostics.push(
                        Diagnostic::error(format!("cannot compare `{}` with `{}`", left_type, right_type))
                            .with_code(ErrorCode::TYPE_MISMATCH)
                            .with_label(span, "incompatible types")
                    );
                }
                MendesType::Bool
            }

            // Logical
            BinOp::And | BinOp::Or => {
                if !matches!((&left_type, &right_type),
                    (MendesType::Bool, MendesType::Bool) |
                    (MendesType::Unknown, _) | (_, MendesType::Unknown)
                ) {
                    self.diagnostics.push(
                        Diagnostic::error(format!("logical operators require `bool`, found `{}` and `{}`", left_type, right_type))
                            .with_code(ErrorCode::TYPE_MISMATCH)
                            .with_label(span, "expected bool")
                    );
                }
                MendesType::Bool
            }

            // Assignment
            BinOp::Assign | BinOp::AddAssign | BinOp::SubAssign | BinOp::MulAssign | BinOp::DivAssign => {
                if !left_type.is_compatible_with(&right_type) {
                    self.diagnostics.push(
                        Diagnostic::error(format!("cannot assign `{}` to `{}`", right_type, left_type))
                            .with_code(ErrorCode::TYPE_MISMATCH)
                            .with_label(span, "incompatible types")
                    );
                }
                MendesType::Unit
            }
        }
    }

    /// Checks unary operation
    fn check_unary(&mut self, op: UnaryOp, expr: &Expr, span: Span) -> MendesType {
        let expr_type = self.check_expr(expr);

        match op {
            UnaryOp::Neg => {
                if !matches!(expr_type, MendesType::Int | MendesType::Float | MendesType::Unknown) {
                    self.diagnostics.push(
                        Diagnostic::error(format!("cannot negate `{}`", expr_type))
                            .with_code(ErrorCode::TYPE_MISMATCH)
                            .with_label(span, "expected int or float")
                    );
                }
                expr_type
            }
            UnaryOp::Not => {
                if !matches!(expr_type, MendesType::Bool | MendesType::Unknown) {
                    self.diagnostics.push(
                        Diagnostic::error(format!("`not` requires `bool`, found `{}`", expr_type))
                            .with_code(ErrorCode::TYPE_MISMATCH)
                            .with_label(span, "expected bool")
                    );
                }
                MendesType::Bool
            }
        }
    }

    /// Checks function call
    fn check_call(&mut self, func: &Expr, args: &[Expr], span: Span) -> MendesType {
        // First, get the argument types
        let arg_types: Vec<MendesType> = args.iter().map(|a| self.check_expr(a)).collect();

        // Try to get generic params if this is a named function
        let (generic_params, func_params, func_ret) = if let Expr::Ident(func_name, _) = func {
            if let Some(symbol) = self.ctx.symbols.lookup(func_name) {
                if let SymbolKind::Function { generic_params, params, return_type, .. } = &symbol.kind {
                    (generic_params.clone(), params.clone(), return_type.clone())
                } else {
                    (vec![], vec![], MendesType::Unknown)
                }
            } else {
                (vec![], vec![], MendesType::Unknown)
            }
        } else {
            // For non-identifier function expressions, fall back to type checking
            let func_type = self.check_expr(func);
            match func_type {
                MendesType::Function { params, ret } => {
                    let param_types: Vec<_> = params.iter().enumerate()
                        .map(|(i, t)| (format!("__arg{}", i), t.clone()))
                        .collect();
                    (vec![], param_types, *ret)
                }
                _ => (vec![], vec![], MendesType::Unknown)
            }
        };

        // Check argument count
        if !func_params.is_empty() && args.len() != func_params.len() {
            self.diagnostics.push(
                Diagnostic::error(format!("expected {} arguments, found {}", func_params.len(), args.len()))
                    .with_code(ErrorCode::TYPE_MISMATCH)
                    .with_label(span, "incorrect number of arguments")
            );
            return MendesType::Unknown;
        }

        // Infer generic type parameters from arguments
        let type_substitutions = self.infer_generic_types(&generic_params, &func_params, &arg_types);

        // Check argument types with substitutions applied
        for (i, ((_, expected_type), arg_type)) in func_params.iter().zip(arg_types.iter()).enumerate() {
            let substituted = self.substitute_generics(expected_type, &type_substitutions);
            if !substituted.is_compatible_with(arg_type) {
                self.diagnostics.push(
                    Diagnostic::error(format!("incompatible argument: expected `{}`, found `{}`", substituted, arg_type))
                        .with_code(ErrorCode::TYPE_MISMATCH)
                        .with_label(args[i].span(), "incompatible type")
                );
            }

            // Check argument move
            if let Expr::Ident(name, arg_span) = &args[i] {
                if !expected_type.is_ref() && !arg_type.is_copy() {
                    self.ownership.mark_moved(name, *arg_span);
                }
            }
        }

        // Apply substitutions to return type
        self.substitute_generics(&func_ret, &type_substitutions)
    }

    /// Infer generic type parameters from argument types
    fn infer_generic_types(
        &self,
        generic_params: &[String],
        func_params: &[(String, MendesType)],
        arg_types: &[MendesType],
    ) -> std::collections::HashMap<String, MendesType> {
        let mut substitutions = std::collections::HashMap::new();

        for ((_param_name, param_type), arg_type) in func_params.iter().zip(arg_types.iter()) {
            self.collect_type_substitutions(param_type, arg_type, generic_params, &mut substitutions);
        }

        substitutions
    }

    /// Collect type substitutions by matching parameter types to argument types
    fn collect_type_substitutions(
        &self,
        param_type: &MendesType,
        arg_type: &MendesType,
        generic_params: &[String],
        substitutions: &mut std::collections::HashMap<String, MendesType>,
    ) {
        match param_type {
            MendesType::Named(name) if generic_params.contains(name) => {
                // This is a generic type parameter, record the substitution
                if !substitutions.contains_key(name) {
                    substitutions.insert(name.clone(), arg_type.clone());
                }
            }
            MendesType::Ref(inner) => {
                if let MendesType::Ref(arg_inner) = arg_type {
                    self.collect_type_substitutions(inner, arg_inner, generic_params, substitutions);
                }
            }
            MendesType::MutRef(inner) => {
                if let MendesType::MutRef(arg_inner) = arg_type {
                    self.collect_type_substitutions(inner, arg_inner, generic_params, substitutions);
                }
            }
            MendesType::Array(inner) => {
                if let MendesType::Array(arg_inner) = arg_type {
                    self.collect_type_substitutions(inner, arg_inner, generic_params, substitutions);
                }
            }
            MendesType::Generic { name: _, args } => {
                if let MendesType::Generic { name: _, args: arg_args } = arg_type {
                    for (param_arg, arg_arg) in args.iter().zip(arg_args.iter()) {
                        self.collect_type_substitutions(param_arg, arg_arg, generic_params, substitutions);
                    }
                }
            }
            MendesType::Tuple(types) => {
                if let MendesType::Tuple(arg_types) = arg_type {
                    for (pt, at) in types.iter().zip(arg_types.iter()) {
                        self.collect_type_substitutions(pt, at, generic_params, substitutions);
                    }
                }
            }
            _ => {}
        }
    }

    /// Substitute generic type parameters with their inferred types
    fn substitute_generics(
        &self,
        ty: &MendesType,
        substitutions: &std::collections::HashMap<String, MendesType>,
    ) -> MendesType {
        match ty {
            MendesType::Named(name) => {
                if let Some(subst) = substitutions.get(name) {
                    subst.clone()
                } else {
                    ty.clone()
                }
            }
            MendesType::Ref(inner) => {
                MendesType::Ref(Box::new(self.substitute_generics(inner, substitutions)))
            }
            MendesType::MutRef(inner) => {
                MendesType::MutRef(Box::new(self.substitute_generics(inner, substitutions)))
            }
            MendesType::Array(inner) => {
                MendesType::Array(Box::new(self.substitute_generics(inner, substitutions)))
            }
            MendesType::Generic { name, args } => {
                MendesType::Generic {
                    name: name.clone(),
                    args: args.iter().map(|a| self.substitute_generics(a, substitutions)).collect(),
                }
            }
            MendesType::Function { params, ret } => {
                MendesType::Function {
                    params: params.iter().map(|p| self.substitute_generics(p, substitutions)).collect(),
                    ret: Box::new(self.substitute_generics(ret, substitutions)),
                }
            }
            MendesType::Future(inner) => {
                MendesType::Future(Box::new(self.substitute_generics(inner, substitutions)))
            }
            MendesType::Tuple(types) => {
                MendesType::Tuple(types.iter().map(|t| self.substitute_generics(t, substitutions)).collect())
            }
            MendesType::Range(inner) => {
                MendesType::Range(Box::new(self.substitute_generics(inner, substitutions)))
            }
            _ => ty.clone(),
        }
    }

    /// Checks field access
    fn check_field_access(&mut self, object: &Expr, field: &str, _span: Span) -> MendesType {
        let object_type = self.check_expr(object);

        match &object_type {
            MendesType::Named(name) => {
                if let Some(struct_def) = self.ctx.types.get_struct(name) {
                    for (field_name, field_type) in &struct_def.fields {
                        if field_name == field {
                            return field_type.clone();
                        }
                    }
                }
                // Field not found or method
                MendesType::Unknown
            }
            MendesType::Ref(inner) | MendesType::MutRef(inner) => {
                // Auto-deref
                if let MendesType::Named(name) = inner.as_ref() {
                    if let Some(struct_def) = self.ctx.types.get_struct(name) {
                        for (field_name, field_type) in &struct_def.fields {
                            if field_name == field {
                                return field_type.clone();
                            }
                        }
                    }
                }
                MendesType::Unknown
            }
            _ => MendesType::Unknown,
        }
    }

    /// Checks method call
    fn check_method_call(&mut self, object: &Expr, method: &str, args: &[Expr], span: Span) -> MendesType {
        let object_type = self.check_expr(object);

        // Get the struct name (handle references)
        let struct_name = match &object_type {
            MendesType::Named(name) => Some(name.clone()),
            MendesType::Ref(inner) | MendesType::MutRef(inner) => {
                if let MendesType::Named(name) = inner.as_ref() {
                    Some(name.clone())
                } else {
                    None
                }
            }
            _ => None,
        };

        if let Some(name) = struct_name {
            // Clone to avoid borrow conflict
            let struct_def = self.ctx.types.get_struct(&name).cloned();

            if let Some(def) = struct_def {
                // Look for the method
                for (method_name, params, return_type, _is_async) in &def.methods {
                    if method_name == method {
                        // Check argument count
                        if args.len() != params.len() {
                            self.diagnostics.push(
                                Diagnostic::error(format!(
                                    "method `{}` expects {} arguments, found {}",
                                    method, params.len(), args.len()
                                ))
                                .with_code(ErrorCode::TYPE_MISMATCH)
                                .with_label(span, "incorrect number of arguments")
                            );
                        }

                        // Check argument types
                        for (arg, (_, expected_type)) in args.iter().zip(params.iter()) {
                            let arg_type = self.check_expr(arg);
                            if !expected_type.is_compatible_with(&arg_type) {
                                self.diagnostics.push(
                                    Diagnostic::error(format!(
                                        "incompatible argument: expected `{}`, found `{}`",
                                        expected_type, arg_type
                                    ))
                                    .with_code(ErrorCode::TYPE_MISMATCH)
                                    .with_label(arg.span(), "incompatible type")
                                );
                            }
                        }

                        return return_type.clone();
                    }
                }

                // Method not found - check if it's a builtin method
                return self.check_builtin_method(&object_type, method, args, span);
            }
        }

        // Check builtin methods for primitive types
        self.check_builtin_method(&object_type, method, args, span)
    }

    /// Checks builtin methods for types
    fn check_builtin_method(&mut self, object_type: &MendesType, method: &str, args: &[Expr], span: Span) -> MendesType {
        // Check args anyway
        for arg in args {
            self.check_expr(arg);
        }

        match object_type {
            MendesType::String => {
                match method {
                    "len" => MendesType::Int,
                    "contains" => MendesType::Bool,
                    "concat" => MendesType::String,
                    "to_upper" | "to_lower" | "trim" => MendesType::String,
                    "split" => MendesType::Array(Box::new(MendesType::String)),
                    _ => MendesType::Unknown,
                }
            }
            MendesType::Array(_) => {
                match method {
                    "len" => MendesType::Int,
                    "push" | "pop" => MendesType::Unit,
                    "is_empty" => MendesType::Bool,
                    _ => MendesType::Unknown,
                }
            }
            MendesType::Generic { name, args: type_args } => {
                match name.as_str() {
                    "Result" => {
                        match method {
                            "is_ok" | "is_err" => MendesType::Bool,
                            "unwrap" => type_args.first().cloned().unwrap_or(MendesType::Unknown),
                            "unwrap_or" => type_args.first().cloned().unwrap_or(MendesType::Unknown),
                            "map" => MendesType::Unknown, // Would need closure support
                            _ => MendesType::Unknown,
                        }
                    }
                    "Option" => {
                        match method {
                            "is_some" | "is_none" => MendesType::Bool,
                            "unwrap" => type_args.first().cloned().unwrap_or(MendesType::Unknown),
                            "unwrap_or" => type_args.first().cloned().unwrap_or(MendesType::Unknown),
                            "map" => MendesType::Unknown, // Would need closure support
                            _ => MendesType::Unknown,
                        }
                    }
                    _ => MendesType::Unknown,
                }
            }
            _ => {
                if method != "to_string" && method != "clone" {
                    self.diagnostics.push(
                        Diagnostic::error(format!("method `{}` not found on type `{}`", method, object_type))
                            .with_code(ErrorCode::UNKNOWN_VARIABLE)
                            .with_label(span, "method not found")
                    );
                }
                MendesType::Unknown
            }
        }
    }

    /// Checks index access
    fn check_index(&mut self, object: &Expr, index: &Expr, span: Span) -> MendesType {
        let object_type = self.check_expr(object);
        let index_type = self.check_expr(index);

        if !matches!(index_type, MendesType::Int | MendesType::Unknown) {
            self.diagnostics.push(
                Diagnostic::error(format!("index must be `int`, found `{}`", index_type))
                    .with_code(ErrorCode::TYPE_MISMATCH)
                    .with_label(index.span(), "expected int")
            );
        }

        match object_type {
            MendesType::Array(inner) => *inner,
            MendesType::String => MendesType::String, // string[i] returns string (char)
            MendesType::Unknown => MendesType::Unknown,
            _ => {
                self.diagnostics.push(
                    Diagnostic::error(format!("type `{}` does not support indexing", object_type))
                        .with_code(ErrorCode::TYPE_MISMATCH)
                        .with_label(span, "not indexable")
                );
                MendesType::Unknown
            }
        }
    }

    /// Checks struct literal
    fn check_struct_lit(&mut self, name: &str, fields: &[(String, Expr)], span: Span) -> MendesType {
        // Clone struct fields to avoid borrow conflict
        let struct_fields = self.ctx.types.get_struct(name).map(|s| s.fields.clone());

        if let Some(expected_fields_vec) = struct_fields {
            let expected_fields: std::collections::HashSet<_> = expected_fields_vec.iter()
                .map(|(n, _)| n.as_str())
                .collect();

            let provided_fields: std::collections::HashSet<_> = fields.iter()
                .map(|(n, _)| n.as_str())
                .collect();

            // Check missing fields
            for field_name in expected_fields.difference(&provided_fields) {
                self.diagnostics.push(
                    Diagnostic::error(format!("field `{}` missing in `{}`", field_name, name))
                        .with_code(ErrorCode::TYPE_MISMATCH)
                        .with_label(span, "required field not provided")
                );
            }

            // Check extra fields
            for field_name in provided_fields.difference(&expected_fields) {
                self.diagnostics.push(
                    Diagnostic::error(format!("field `{}` does not exist in `{}`", field_name, name))
                        .with_code(ErrorCode::TYPE_MISMATCH)
                        .with_label(span, "unknown field")
                );
            }

            // Check field types
            for (field_name, field_expr) in fields {
                let field_type = self.check_expr(field_expr);
                if let Some((_, expected_type)) = expected_fields_vec.iter().find(|(n, _)| n == field_name) {
                    if !expected_type.is_compatible_with(&field_type) {
                        self.diagnostics.push(
                            Diagnostic::error(format!(
                                "incompatible type for field `{}`: expected `{}`, found `{}`",
                                field_name, expected_type, field_type
                            ))
                            .with_code(ErrorCode::TYPE_MISMATCH)
                            .with_label(field_expr.span(), "incompatible type")
                        );
                    }
                }
            }

            MendesType::Named(name.to_string())
        } else {
            self.diagnostics.push(
                Diagnostic::error(format!("struct `{}` not found", name))
                    .with_code(ErrorCode::UNKNOWN_TYPE)
                    .with_label(span, "type not declared")
            );
            MendesType::Unknown
        }
    }
}

/// Helper function for analysis
pub fn analyze(program: &Program, ctx: &mut SemanticContext) -> Diagnostics {
    let mut checker = TypeChecker::new(ctx);
    checker.check_program(program);
    checker.take_diagnostics()
}

#[cfg(test)]
mod tests {
    use super::*;
    use mendes_lexer::Lexer;
    use mendes_parser::parse;

    fn analyze_source(source: &str) -> Diagnostics {
        let mut lexer = Lexer::new(source, 0);
        let tokens = lexer.tokenize();
        let (program, _) = parse(tokens);
        let mut ctx = SemanticContext::new();
        analyze(&program, &mut ctx)
    }

    #[test]
    fn test_type_mismatch() {
        let diags = analyze_source("let x: int = \"hello\"\n");
        assert!(diags.has_errors());
    }

    #[test]
    fn test_valid_let() {
        let diags = analyze_source("let x: int = 10\n");
        assert!(!diags.has_errors());
    }

    #[test]
    fn test_unknown_variable() {
        let diags = analyze_source("let x: int = y\n");
        assert!(diags.has_errors());
    }

    #[test]
    fn test_closure_basic() {
        let diags = analyze_source("let add = |x: int, y: int| x + y\n");
        assert!(!diags.has_errors());
    }

    #[test]
    fn test_closure_with_return_type() {
        let diags = analyze_source("let double = |n: int| -> int: n * 2\n");
        assert!(!diags.has_errors());
    }

    #[test]
    fn test_higher_order_function() {
        let source = r#"
fn apply(f: fn(int) -> int, x: int) -> int:
    return f(x)
"#;
        let diags = analyze_source(source);
        assert!(!diags.has_errors());
    }

    #[test]
    fn test_closure_type_compatibility() {
        let source = r#"
fn apply(f: fn(int) -> int, x: int) -> int:
    return f(x)

fn test() -> int:
    let square = |n: int| n * n
    return apply(square, 5)
"#;
        let diags = analyze_source(source);
        assert!(!diags.has_errors());
    }
}
