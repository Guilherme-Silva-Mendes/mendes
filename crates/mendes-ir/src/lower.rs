//! Lowering AST → IR
//!
//! Converts the high-level AST to intermediate representation.

use crate::types::{IrType, StructDef, GenericParam};
use crate::instruction::{Instruction, Value, BinaryOp, CompareOp};
use crate::module::{Module, Function, HttpRoute, WsRoute, ServerConfig, DatabaseConfig, TraitDef, TraitMethodDef, ImplDef, TypeAlias};
use mendes_parser::*;
use std::collections::HashMap;

/// Lowering context
pub struct LoweringContext {
    /// Module being built
    pub module: Module,
    /// Map of local variables to their names in IR (reserved for future use)
    #[allow(dead_code)]
    locals: HashMap<String, String>,
    /// Label counter for blocks
    label_counter: u32,
    /// HTTP handler counter
    handler_counter: u32,
}

impl LoweringContext {
    pub fn new(module_name: impl Into<String>) -> Self {
        Self {
            module: Module::new(module_name),
            locals: HashMap::new(),
            label_counter: 0,
            handler_counter: 0,
        }
    }

    /// Generates a unique label
    fn new_label(&mut self, prefix: &str) -> String {
        let label = format!("{}_{}", prefix, self.label_counter);
        self.label_counter += 1;
        label
    }

    /// Generates a unique name for HTTP handler
    fn new_handler_name(&mut self, method: &str, path: &str) -> String {
        let safe_path = path.replace('/', "_").replace('{', "").replace('}', "").replace(':', "_");
        let name = format!("__http_{}_{}{}", method.to_lowercase(), safe_path, self.handler_counter);
        self.handler_counter += 1;
        name
    }
}

/// Converts an AST program to an IR module
pub fn lower_program(program: &Program) -> Module {
    let mut ctx = LoweringContext::new("main");

    // First pass: collect declarations
    for stmt in &program.statements {
        collect_declarations(&mut ctx, stmt);
    }

    // Second pass: generate code
    for stmt in &program.statements {
        lower_statement(&mut ctx, stmt);
    }

    ctx.module
}

/// Converts AST GenericParam to IR GenericParam
fn convert_generic_param(ast_param: &mendes_parser::GenericParam) -> GenericParam {
    GenericParam {
        name: ast_param.name.clone(),
        bounds: ast_param.bounds.clone(),
    }
}

/// Collects declarations (structs, functions, etc)
fn collect_declarations(ctx: &mut LoweringContext, stmt: &Stmt) {
    match stmt {
        Stmt::Struct(s) => {
            let mut def = StructDef::new(s.name.clone());
            // Add generic parameters
            for gp in &s.generic_params {
                def.add_generic_param(convert_generic_param(gp));
            }
            for field in &s.fields {
                def.add_field(field.name.clone(), IrType::from_mendes_type(&field.ty));
            }
            // Register method names
            for method in &s.methods {
                def.add_method(format!("{}::{}", s.name, method.name));
            }
            ctx.module.add_struct(def);
        }
        Stmt::Trait(t) => {
            let mut def = TraitDef::new(t.name.clone());
            // Add generic parameters
            for gp in &t.generic_params {
                def.add_generic_param(convert_generic_param(gp));
            }
            // Add method signatures
            for method in &t.methods {
                let return_type = method.return_type.as_ref()
                    .map(IrType::from_mendes_type)
                    .unwrap_or(IrType::Void);
                let mut method_def = TraitMethodDef::new(&method.name, return_type);
                method_def.is_async = method.is_async;
                method_def.receiver = match method.receiver {
                    MethodReceiver::Ref => 0,
                    MethodReceiver::MutRef => 1,
                    MethodReceiver::Value => 2,
                };
                for param in &method.params {
                    method_def.add_param(&param.name, IrType::from_mendes_type(&param.ty));
                }
                def.add_method(method_def);
            }
            ctx.module.add_trait(def);
        }
        Stmt::TypeAlias { name, ty, span: _ } => {
            let alias = TypeAlias::new(name.clone(), IrType::from_mendes_type(ty));
            ctx.module.add_type_alias(alias);
        }
        Stmt::Server(s) => {
            ctx.module.server = Some(ServerConfig {
                host: s.host.clone(),
                port: s.port,
            });
        }
        Stmt::Db(db) => {
            let db_type = match db.db_type {
                DbType::Postgres => "postgres",
                DbType::Mysql => "mysql",
                DbType::Sqlite => "sqlite",
            };
            ctx.module.databases.push(DatabaseConfig {
                name: db.name.clone(),
                db_type: db_type.to_string(),
                url: db.url.clone(),
                pool_size: db.pool_size,
            });
        }
        Stmt::Middleware(m) => {
            ctx.module.middlewares.push(m.name.clone());
        }
        _ => {}
    }
}

/// Converts a statement to IR
fn lower_statement(ctx: &mut LoweringContext, stmt: &Stmt) {
    match stmt {
        Stmt::Fn(f) => {
            lower_function(ctx, f);
        }
        Stmt::Struct(s) => {
            // Lower struct methods
            for method in &s.methods {
                lower_method(ctx, &s.name, method, &s.generic_params);
            }
        }
        Stmt::Trait(_) => {
            // Trait declarations are already handled in collect_declarations
            // No code generation needed for trait definitions themselves
        }
        Stmt::ImplTrait(impl_decl) => {
            lower_impl_trait(ctx, impl_decl);
        }
        Stmt::TypeAlias { .. } => {
            // Type aliases are already handled in collect_declarations
        }
        Stmt::Api(api) => {
            lower_api(ctx, api);
        }
        Stmt::WebSocket(ws) => {
            lower_websocket(ctx, ws);
        }
        Stmt::Middleware(m) => {
            lower_middleware(ctx, m);
        }
        // Server, Db have already been processed
        _ => {}
    }
}

/// Converts a function to IR
fn lower_function(ctx: &mut LoweringContext, f: &FnDecl) {
    let return_type = f.return_type.as_ref()
        .map(IrType::from_mendes_type)
        .unwrap_or(IrType::Void);

    let mut func = Function::new(&f.name, return_type, f.is_async);

    // Generic parameters
    for gp in &f.generic_params {
        func.add_generic_param(convert_generic_param(gp));
    }

    // Parameters
    for param in &f.params {
        func.add_param(&param.name, IrType::from_mendes_type(&param.ty));
    }

    // Body
    let mut lowerer = FunctionLowerer::new(ctx, &mut func);
    for stmt in &f.body {
        lowerer.lower_stmt(stmt);
    }

    // Ensure there is a return
    if !func.current_block().is_terminated() {
        func.emit(Instruction::Return(Value::Void));
    }

    ctx.module.add_function(func);
}

/// Converts an API to IR (generates handler function)
fn lower_api(ctx: &mut LoweringContext, api: &ApiDecl) {
    let method = match api.method {
        HttpMethod::Get => "GET",
        HttpMethod::Post => "POST",
        HttpMethod::Put => "PUT",
        HttpMethod::Delete => "DELETE",
        HttpMethod::Patch => "PATCH",
    };

    let handler_name = ctx.new_handler_name(method, &api.path);

    let return_type = api.return_type.as_ref()
        .map(IrType::from_mendes_type)
        .unwrap_or(IrType::Void);

    let mut func = Function::new(&handler_name, return_type, api.is_async);

    // Implicit API parameters
    func.add_param("__request", IrType::Struct("Request".to_string()));

    // Path parameters (extracted from path like /users/{id:int})
    extract_path_params(&api.path, &mut func);

    // Body type if specified
    if api.body_type.is_some() {
        func.add_param("body", IrType::Struct("Body".to_string()));
    }

    // Handler body
    let mut lowerer = FunctionLowerer::new(ctx, &mut func);
    for stmt in &api.handler {
        lowerer.lower_stmt(stmt);
    }

    // Ensure there is a return
    if !func.current_block().is_terminated() {
        func.emit(Instruction::Return(Value::Void));
    }

    ctx.module.add_function(func);

    // Register route
    let mut route = HttpRoute::new(method, &api.path, &handler_name);
    route.middlewares = api.middlewares.clone();
    route.is_async = api.is_async;
    ctx.module.add_route(route);
}

/// Converts a WebSocket declaration to IR
fn lower_websocket(ctx: &mut LoweringContext, ws: &WsDecl) {
    // Create handler for on_connect
    if let Some(handler) = &ws.on_connect {
        let handler_name = format!("__ws_connect__{}", ws.path.replace('/', "_"));
        let mut func = Function::new(&handler_name, IrType::Void, true);
        func.add_param("conn", IrType::Struct("WsConnection".to_string()));
        extract_path_params(&ws.path, &mut func);

        let mut lowerer = FunctionLowerer::new(ctx, &mut func);
        for stmt in handler {
            lowerer.lower_stmt(stmt);
        }
        if !func.current_block().is_terminated() {
            func.emit(Instruction::Return(Value::Void));
        }
        ctx.module.add_function(func);
    }

    // Create handler for on_message
    if let Some(handler) = &ws.on_message {
        let handler_name = format!("__ws_message__{}", ws.path.replace('/', "_"));
        let mut func = Function::new(&handler_name, IrType::Void, true);
        func.add_param("conn", IrType::Struct("WsConnection".to_string()));
        func.add_param("message", IrType::String);
        extract_path_params(&ws.path, &mut func);

        let mut lowerer = FunctionLowerer::new(ctx, &mut func);
        for stmt in handler {
            lowerer.lower_stmt(stmt);
        }
        if !func.current_block().is_terminated() {
            func.emit(Instruction::Return(Value::Void));
        }
        ctx.module.add_function(func);
    }

    // Create handler for on_disconnect
    if let Some(handler) = &ws.on_disconnect {
        let handler_name = format!("__ws_disconnect__{}", ws.path.replace('/', "_"));
        let mut func = Function::new(&handler_name, IrType::Void, true);
        func.add_param("conn", IrType::Struct("WsConnection".to_string()));
        extract_path_params(&ws.path, &mut func);

        let mut lowerer = FunctionLowerer::new(ctx, &mut func);
        for stmt in handler {
            lowerer.lower_stmt(stmt);
        }
        if !func.current_block().is_terminated() {
            func.emit(Instruction::Return(Value::Void));
        }
        ctx.module.add_function(func);
    }

    // Register WebSocket route
    ctx.module.add_websocket_route(WsRoute {
        path: ws.path.clone(),
        on_connect: ws.on_connect.as_ref().map(|_| format!("__ws_connect__{}", ws.path.replace('/', "_"))),
        on_message: ws.on_message.as_ref().map(|_| format!("__ws_message__{}", ws.path.replace('/', "_"))),
        on_disconnect: ws.on_disconnect.as_ref().map(|_| format!("__ws_disconnect__{}", ws.path.replace('/', "_"))),
        middlewares: ws.middlewares.clone(),
    });
}

/// Extracts parameters from path (/users/{id:int})
fn extract_path_params(path: &str, func: &mut Function) {
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
                    "int" => IrType::I64,
                    "string" | "" => IrType::String,
                    _ => IrType::String,
                };
                func.add_param(&param, ty);
            }
        }
    }
}

/// Converts a method to IR
fn lower_method(ctx: &mut LoweringContext, struct_name: &str, m: &MethodDecl, struct_generic_params: &[mendes_parser::GenericParam]) {
    let return_type = m.return_type.as_ref()
        .map(IrType::from_mendes_type)
        .unwrap_or(IrType::Void);

    // Method name is StructName::method_name
    let method_name = format!("{}::{}", struct_name, m.name);
    let mut func = Function::new(&method_name, return_type, m.is_async);

    // Inherit generic parameters from struct
    for gp in struct_generic_params {
        func.add_generic_param(convert_generic_param(gp));
    }

    // First parameter is always self (pointer to struct)
    func.add_param("self", IrType::Ptr(Box::new(IrType::Struct(struct_name.to_string()))));

    // Other parameters
    for param in &m.params {
        func.add_param(&param.name, IrType::from_mendes_type(&param.ty));
    }

    // Body
    let mut lowerer = FunctionLowerer::new(ctx, &mut func);
    for stmt in &m.body {
        lowerer.lower_stmt(stmt);
    }

    // Ensure there is a return
    if !func.current_block().is_terminated() {
        func.emit(Instruction::Return(Value::Void));
    }

    ctx.module.add_function(func);
}

/// Converts a trait implementation to IR
fn lower_impl_trait(ctx: &mut LoweringContext, impl_decl: &ImplTraitDecl) {
    // Create the ImplDef
    let mut impl_def = ImplDef::new(&impl_decl.trait_name, &impl_decl.type_name);

    // Add generic parameters
    for gp in &impl_decl.generic_params {
        impl_def.generic_params.push(convert_generic_param(gp));
    }

    // Lower each method
    for method in &impl_decl.methods {
        // Method name follows pattern: TypeName::TraitName::method_name
        let method_name = format!("<{} as {}>::{}", impl_decl.type_name, impl_decl.trait_name, method.name);
        impl_def.add_method(method_name.clone());

        let return_type = method.return_type.as_ref()
            .map(IrType::from_mendes_type)
            .unwrap_or(IrType::Void);

        let mut func = Function::new(&method_name, return_type, method.is_async);

        // Add generic parameters
        for gp in &impl_decl.generic_params {
            func.add_generic_param(convert_generic_param(gp));
        }

        // First parameter is always self
        let self_type = IrType::Ptr(Box::new(IrType::Struct(impl_decl.type_name.clone())));
        func.add_param("self", self_type);

        // Other parameters
        for param in &method.params {
            func.add_param(&param.name, IrType::from_mendes_type(&param.ty));
        }

        // Body
        let mut lowerer = FunctionLowerer::new(ctx, &mut func);
        for stmt in &method.body {
            lowerer.lower_stmt(stmt);
        }

        if !func.current_block().is_terminated() {
            func.emit(Instruction::Return(Value::Void));
        }

        ctx.module.add_function(func);
    }

    ctx.module.add_impl(impl_def);
}

/// Converts middleware to IR
fn lower_middleware(ctx: &mut LoweringContext, m: &MiddlewareDecl) {
    let mut func = Function::new(
        format!("__middleware_{}", m.name),
        IrType::Struct("MiddlewareResult".to_string()),
        false,
    );

    func.add_param("request", IrType::Struct("Request".to_string()));

    let mut lowerer = FunctionLowerer::new(ctx, &mut func);
    for stmt in &m.body {
        lowerer.lower_stmt(stmt);
    }

    if !func.current_block().is_terminated() {
        func.emit(Instruction::Return(Value::Void));
    }

    ctx.module.add_function(func);
}

/// Function lowerer - converts statements and expressions within a function
struct FunctionLowerer<'a, 'b> {
    ctx: &'a mut LoweringContext,
    func: &'b mut Function,
    /// Map of variables to values
    vars: HashMap<String, Value>,
}

impl<'a, 'b> FunctionLowerer<'a, 'b> {
    fn new(ctx: &'a mut LoweringContext, func: &'b mut Function) -> Self {
        Self {
            ctx,
            func,
            vars: HashMap::new(),
        }
    }

    /// Generates a unique label
    fn new_label(&mut self, prefix: &str) -> String {
        self.ctx.new_label(prefix)
    }

    /// Infers the IR type from an expression (used for type inference in let statements)
    fn infer_expr_type(&self, expr: &Expr) -> Option<IrType> {
        match expr {
            Expr::Closure { params, return_type, body: _, span: _ } => {
                // Build function type from closure
                let param_types: Vec<IrType> = params.iter()
                    .map(|p| p.ty.as_ref()
                        .map(IrType::from_mendes_type)
                        .unwrap_or(IrType::I64))
                    .collect();

                let ret_type = return_type.as_ref()
                    .map(IrType::from_mendes_type)
                    .unwrap_or(IrType::I64);

                Some(IrType::Function {
                    params: param_types,
                    ret: Box::new(ret_type),
                })
            }
            Expr::IntLit(_, _) => Some(IrType::I64),
            Expr::FloatLit(_, _) => Some(IrType::F64),
            Expr::BoolLit(_, _) => Some(IrType::Bool),
            Expr::StringLit(_, _) => Some(IrType::String),
            Expr::ArrayLit(elements, _) => {
                // Infer element type from first element
                if let Some(first) = elements.first() {
                    let elem_type = self.infer_expr_type(first).unwrap_or(IrType::I64);
                    Some(IrType::Array(Box::new(elem_type), elements.len()))
                } else {
                    Some(IrType::Array(Box::new(IrType::I64), 0))
                }
            }
            _ => None, // Fall back to default type
        }
    }

    /// Converts statement
    fn lower_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let { name, ty, value, mutable: _, span: _ } => {
                // Infer type from expression if no type annotation
                let ir_ty = ty.as_ref()
                    .map(IrType::from_mendes_type)
                    .or_else(|| self.infer_expr_type(value))
                    .unwrap_or(IrType::I64);

                // Allocate variable
                self.func.add_local(name.clone(), ir_ty.clone());
                self.func.emit(Instruction::Alloca {
                    dest: name.clone(),
                    ty: ir_ty,
                });

                // Evaluate value and store
                let val = self.lower_expr(value);
                self.func.emit(Instruction::Store {
                    value: val.clone(),
                    ptr: Value::Local(name.clone()),
                });

                self.vars.insert(name.clone(), val);
            }

            Stmt::Return { value, span: _ } => {
                let val = value.as_ref()
                    .map(|e| self.lower_expr(e))
                    .unwrap_or(Value::Void);
                self.func.emit(Instruction::Return(val));
            }

            Stmt::If { condition, then_block, else_block, span: _ } => {
                let cond = self.lower_expr(condition);

                let then_label = self.new_label("then");
                let else_label = self.new_label("else");
                let end_label = self.new_label("endif");

                self.func.emit(Instruction::CondBranch {
                    cond,
                    then_label: then_label.clone(),
                    else_label: if else_block.is_some() { else_label.clone() } else { end_label.clone() },
                });

                // Then block
                self.func.new_block(&then_label);
                for s in then_block {
                    self.lower_stmt(s);
                }
                if !self.func.current_block().is_terminated() {
                    self.func.emit(Instruction::Branch { target: end_label.clone() });
                }

                // Else block
                if let Some(else_stmts) = else_block {
                    self.func.new_block(&else_label);
                    for s in else_stmts {
                        self.lower_stmt(s);
                    }
                    if !self.func.current_block().is_terminated() {
                        self.func.emit(Instruction::Branch { target: end_label.clone() });
                    }
                }

                // End block
                self.func.new_block(&end_label);
            }

            Stmt::While { condition, body, span: _ } => {
                let cond_label = self.new_label("while_cond");
                let body_label = self.new_label("while_body");
                let end_label = self.new_label("while_end");

                self.func.emit(Instruction::Branch { target: cond_label.clone() });

                // Condition block
                self.func.new_block(&cond_label);
                let cond = self.lower_expr(condition);
                self.func.emit(Instruction::CondBranch {
                    cond,
                    then_label: body_label.clone(),
                    else_label: end_label.clone(),
                });

                // Body block
                self.func.new_block(&body_label);
                for s in body {
                    self.lower_stmt(s);
                }
                if !self.func.current_block().is_terminated() {
                    self.func.emit(Instruction::Branch { target: cond_label.clone() });
                }

                // End block
                self.func.new_block(&end_label);
            }

            Stmt::For { var, iter, body, span: _ } => {
                // Handle different iterator types
                match iter {
                    // Range iteration: for i in 0..10 or for i in 0..=10
                    Expr::Range { start, end, inclusive, span: _ } => {
                        self.lower_for_range(var, start, end, *inclusive, body);
                    }
                    // Array iteration: for item in array
                    _ => {
                        self.lower_for_iter(var, iter, body);
                    }
                }
            }

            Stmt::Expr(expr) => {
                self.lower_expr(expr);
            }

            // Other statements are not relevant inside functions
            _ => {}
        }
    }

    /// Lower for loop over a range: for i in start..end or for i in start..=end
    fn lower_for_range(
        &mut self,
        var: &str,
        start: &Option<Box<Expr>>,
        end: &Option<Box<Expr>>,
        inclusive: bool,
        body: &[Stmt],
    ) {
        let loop_id = self.func.new_temp();
        let cond_label = format!("for_cond_{}", loop_id);
        let body_label = format!("for_body_{}", loop_id);
        let inc_label = format!("for_inc_{}", loop_id);
        let end_label = format!("for_end_{}", loop_id);

        // Get start and end values
        let start_val = if let Some(s) = start {
            self.lower_expr(s)
        } else {
            Value::ConstInt(0)
        };

        let end_val = if let Some(e) = end {
            self.lower_expr(e)
        } else {
            // If no end, use a large value (infinite loop until break)
            Value::ConstInt(i64::MAX)
        };

        // Allocate loop variable
        self.func.add_local(var.to_string(), IrType::I64);
        self.func.emit(Instruction::Alloca {
            dest: var.to_string(),
            ty: IrType::I64,
        });

        // Initialize loop variable to start
        self.func.emit(Instruction::Store {
            value: start_val,
            ptr: Value::Local(var.to_string()),
        });
        self.vars.insert(var.to_string(), Value::Local(var.to_string()));

        // Jump to condition
        self.func.emit(Instruction::Branch {
            target: cond_label.clone(),
        });

        // Condition block
        self.func.new_block(&cond_label);
        let current_val = self.func.new_temp();
        self.func.emit(Instruction::Load {
            dest: current_val,
            ptr: Value::Local(var.to_string()),
            ty: IrType::I64,
        });

        // Compare: current < end (or <= for inclusive)
        let cmp_op = if inclusive {
            CompareOp::Le // Less than or equal
        } else {
            CompareOp::Lt // Less than
        };
        let cmp_result = self.func.new_temp();
        self.func.emit(Instruction::Compare {
            dest: cmp_result,
            op: cmp_op,
            left: Value::Temp(current_val),
            right: end_val,
        });

        // Branch based on comparison
        self.func.emit(Instruction::CondBranch {
            cond: Value::Temp(cmp_result),
            then_label: body_label.clone(),
            else_label: end_label.clone(),
        });

        // Body block
        self.func.new_block(&body_label);
        for s in body {
            self.lower_stmt(s);
        }
        // Jump to increment
        self.func.emit(Instruction::Branch {
            target: inc_label.clone(),
        });

        // Increment block
        self.func.new_block(&inc_label);
        let inc_val = self.func.new_temp();
        self.func.emit(Instruction::Load {
            dest: inc_val,
            ptr: Value::Local(var.to_string()),
            ty: IrType::I64,
        });
        let new_val = self.func.new_temp();
        self.func.emit(Instruction::Binary {
            dest: new_val,
            op: BinaryOp::Add,
            left: Value::Temp(inc_val),
            right: Value::ConstInt(1),
        });
        self.func.emit(Instruction::Store {
            value: Value::Temp(new_val),
            ptr: Value::Local(var.to_string()),
        });
        // Jump back to condition
        self.func.emit(Instruction::Branch {
            target: cond_label,
        });

        // End block
        self.func.new_block(&end_label);
    }

    /// Lower for loop over an iterable (array, etc.)
    fn lower_for_iter(&mut self, var: &str, iter: &Expr, body: &[Stmt]) {
        let loop_id = self.func.new_temp();
        let cond_label = format!("for_cond_{}", loop_id);
        let body_label = format!("for_body_{}", loop_id);
        let inc_label = format!("for_inc_{}", loop_id);
        let end_label = format!("for_end_{}", loop_id);

        // Evaluate the iterator
        let iter_val = self.lower_expr(iter);

        // Allocate index variable (__idx)
        let idx_var = format!("__idx_{}", loop_id);
        self.func.add_local(idx_var.clone(), IrType::I64);
        self.func.emit(Instruction::Alloca {
            dest: idx_var.clone(),
            ty: IrType::I64,
        });
        self.func.emit(Instruction::Store {
            value: Value::ConstInt(0),
            ptr: Value::Local(idx_var.clone()),
        });

        // Allocate loop variable
        self.func.add_local(var.to_string(), IrType::I64); // Simplified to i64
        self.func.emit(Instruction::Alloca {
            dest: var.to_string(),
            ty: IrType::I64,
        });
        self.vars.insert(var.to_string(), Value::Local(var.to_string()));

        // Get array length (simplified - assumes array has .len() method or use a builtin)
        let len_temp = self.func.new_temp();
        self.func.emit(Instruction::Call {
            dest: Some(len_temp),
            func: "len".to_string(),
            args: vec![iter_val.clone()],
        });

        // Jump to condition
        self.func.emit(Instruction::Branch {
            target: cond_label.clone(),
        });

        // Condition block: check idx < len
        self.func.new_block(&cond_label);
        let current_idx = self.func.new_temp();
        self.func.emit(Instruction::Load {
            dest: current_idx,
            ptr: Value::Local(idx_var.clone()),
            ty: IrType::I64,
        });
        let cmp_result = self.func.new_temp();
        self.func.emit(Instruction::Compare {
            dest: cmp_result,
            op: CompareOp::Lt,
            left: Value::Temp(current_idx),
            right: Value::Temp(len_temp),
        });
        self.func.emit(Instruction::CondBranch {
            cond: Value::Temp(cmp_result),
            then_label: body_label.clone(),
            else_label: end_label.clone(),
        });

        // Body block
        self.func.new_block(&body_label);

        // Get element at index: var = iter[idx]
        let elem_temp = self.func.new_temp();
        self.func.emit(Instruction::GetElement {
            dest: elem_temp,
            ptr: iter_val,
            index: Value::Temp(current_idx),
        });
        self.func.emit(Instruction::Store {
            value: Value::Temp(elem_temp),
            ptr: Value::Local(var.to_string()),
        });

        // Execute body
        for s in body {
            self.lower_stmt(s);
        }

        // Jump to increment
        self.func.emit(Instruction::Branch {
            target: inc_label.clone(),
        });

        // Increment block
        self.func.new_block(&inc_label);
        let inc_idx = self.func.new_temp();
        self.func.emit(Instruction::Load {
            dest: inc_idx,
            ptr: Value::Local(idx_var.clone()),
            ty: IrType::I64,
        });
        let new_idx = self.func.new_temp();
        self.func.emit(Instruction::Binary {
            dest: new_idx,
            op: BinaryOp::Add,
            left: Value::Temp(inc_idx),
            right: Value::ConstInt(1),
        });
        self.func.emit(Instruction::Store {
            value: Value::Temp(new_idx),
            ptr: Value::Local(idx_var.clone()),
        });
        self.func.emit(Instruction::Branch {
            target: cond_label,
        });

        // End block
        self.func.new_block(&end_label);
    }

    /// Converts expression and returns the resulting Value
    fn lower_expr(&mut self, expr: &Expr) -> Value {
        match expr {
            Expr::IntLit(v, _) => Value::ConstInt(*v),

            Expr::FloatLit(v, _) => Value::const_float(*v),

            Expr::BoolLit(v, _) => Value::ConstBool(*v),

            Expr::StringLit(s, _) => {
                let idx = self.ctx.module.add_string(s.clone());
                Value::ConstString(idx)
            }

            Expr::None(_) => Value::Void,

            Expr::Ident(name, _) => {
                // Check if it's a local variable
                if self.vars.contains_key(name) {
                    let temp = self.func.new_temp();
                    self.func.emit(Instruction::Load {
                        dest: temp,
                        ptr: Value::Local(name.clone()),
                        ty: IrType::I64, // Simplificado
                    });
                    Value::Temp(temp)
                } else if self.func.params.iter().any(|(n, _)| n == name) {
                    // Parameters are accessed by name (like local variables)
                    Value::Local(name.clone())
                } else {
                    // Global or unknown
                    Value::Global(name.clone())
                }
            }

            Expr::Binary { left, op, right, span: _ } => {
                let left_val = self.lower_expr(left);
                let right_val = self.lower_expr(right);
                let dest = self.func.new_temp();

                let ir_op = match op {
                    BinOp::Add | BinOp::AddAssign => Some(BinaryOp::Add),
                    BinOp::Sub | BinOp::SubAssign => Some(BinaryOp::Sub),
                    BinOp::Mul | BinOp::MulAssign => Some(BinaryOp::Mul),
                    BinOp::Div | BinOp::DivAssign => Some(BinaryOp::Div),
                    BinOp::Mod => Some(BinaryOp::Mod),
                    BinOp::And => Some(BinaryOp::And),
                    BinOp::Or => Some(BinaryOp::Or),
                    _ => None,
                };

                if let Some(binary_op) = ir_op {
                    self.func.emit(Instruction::Binary {
                        dest,
                        op: binary_op,
                        left: left_val,
                        right: right_val,
                    });
                    return Value::Temp(dest);
                }

                // Comparisons
                let cmp_op = match op {
                    BinOp::Eq => Some(CompareOp::Eq),
                    BinOp::Ne => Some(CompareOp::Ne),
                    BinOp::Lt => Some(CompareOp::Lt),
                    BinOp::Le => Some(CompareOp::Le),
                    BinOp::Gt => Some(CompareOp::Gt),
                    BinOp::Ge => Some(CompareOp::Ge),
                    _ => None,
                };

                if let Some(compare_op) = cmp_op {
                    self.func.emit(Instruction::Compare {
                        dest,
                        op: compare_op,
                        left: left_val,
                        right: right_val,
                    });
                    return Value::Temp(dest);
                }

                // Assignment
                if matches!(op, BinOp::Assign) {
                    if let Expr::Ident(name, _) = left.as_ref() {
                        self.func.emit(Instruction::Store {
                            value: right_val,
                            ptr: Value::Local(name.clone()),
                        });
                        return Value::Void;
                    }
                }

                Value::Void
            }

            Expr::Unary { op, expr, span: _ } => {
                let val = self.lower_expr(expr);
                let dest = self.func.new_temp();

                match op {
                    UnaryOp::Neg => {
                        self.func.emit(Instruction::Neg { dest, value: val });
                    }
                    UnaryOp::Not => {
                        self.func.emit(Instruction::Not { dest, value: val });
                    }
                }

                Value::Temp(dest)
            }

            Expr::Call { func: callee, args, span: _ } => {
                let func_name = match callee.as_ref() {
                    Expr::Ident(name, _) => name.clone(),
                    Expr::FieldAccess { object, field, .. } => {
                        // db.main.query -> __db_main_query
                        let obj_name = match object.as_ref() {
                            Expr::Ident(n, _) => n.clone(),
                            Expr::FieldAccess { field: f, .. } => f.clone(),
                            _ => "unknown".to_string(),
                        };
                        format!("__{}_{}", obj_name, field)
                    }
                    _ => "unknown".to_string(),
                };

                let arg_values: Vec<_> = args.iter().map(|a| self.lower_expr(a)).collect();

                let dest = self.func.new_temp();
                self.func.emit(Instruction::Call {
                    dest: Some(dest),
                    func: func_name,
                    args: arg_values,
                });

                Value::Temp(dest)
            }

            Expr::Await { expr, span: _ } => {
                let future = self.lower_expr(expr);
                let dest = self.func.new_temp();

                self.func.emit(Instruction::Await { dest, future });

                Value::Temp(dest)
            }

            Expr::FieldAccess { object, field, span: _ } => {
                let obj_val = self.lower_expr(object);
                let dest = self.func.new_temp();

                // Try to determine struct name from variable
                let struct_name = match object.as_ref() {
                    Expr::Ident(var_name, _) => {
                        // Look up variable type from function locals or params
                        if let Some(ty) = self.func.locals.get(var_name) {
                            if let IrType::Struct(name) = ty {
                                name.clone()
                            } else {
                                "Unknown".to_string()
                            }
                        } else {
                            "Unknown".to_string()
                        }
                    }
                    _ => "Unknown".to_string(),
                };

                // Try to get field index from struct definition
                let field_index = self.ctx.module.get_struct(&struct_name)
                    .and_then(|s| s.field_index(field))
                    .unwrap_or(0);

                self.func.emit(Instruction::GetField {
                    dest,
                    ptr: obj_val,
                    struct_name,
                    field_index,
                    field_name: field.clone(),
                });

                Value::Temp(dest)
            }

            Expr::Index { object, index, span: _ } => {
                let obj_val = self.lower_expr(object);
                let idx_val = self.lower_expr(index);
                let dest = self.func.new_temp();

                self.func.emit(Instruction::GetElement {
                    dest,
                    ptr: obj_val,
                    index: idx_val,
                });

                Value::Temp(dest)
            }

            Expr::Borrow { expr, mutable: _, span: _ } => {
                // Borrow returns a pointer to the value
                if let Expr::Ident(name, _) = expr.as_ref() {
                    return Value::Local(name.clone());
                }
                self.lower_expr(expr)
            }

            Expr::StructLit { name, fields, span: _ } => {
                let dest = self.func.new_temp();
                self.func.emit(Instruction::NewStruct {
                    dest,
                    struct_name: name.clone(),
                });

                // Initialize fields
                if let Some(struct_def) = self.ctx.module.get_struct(name) {
                    let struct_def = struct_def.clone(); // Clone to avoid borrow conflict
                    for (field_name, field_expr) in fields {
                        let val = self.lower_expr(field_expr);
                        if let Some(idx) = struct_def.field_index(field_name) {
                            self.func.emit(Instruction::SetField {
                                ptr: Value::Temp(dest),
                                struct_name: name.clone(),
                                field_index: idx,
                                field_name: field_name.clone(),
                                value: val,
                            });
                        }
                    }
                }

                Value::Temp(dest)
            }

            Expr::ArrayLit(elements, _) => {
                let dest = self.func.new_temp();
                self.func.emit(Instruction::NewArray {
                    dest,
                    elem_type: IrType::I64, // Simplificado
                    size: Value::ConstInt(elements.len() as i64),
                });

                // Initialize elements
                for (i, elem) in elements.iter().enumerate() {
                    let val = self.lower_expr(elem);
                    self.func.emit(Instruction::SetElement {
                        ptr: Value::Temp(dest),
                        index: Value::ConstInt(i as i64),
                        value: val,
                    });
                }

                Value::Temp(dest)
            }

            Expr::Ok(inner, _) => {
                // Ok(value) -> create Result with tag 0
                let val = self.lower_expr(inner);
                let dest = self.func.new_temp();
                self.func.emit(Instruction::Call {
                    dest: Some(dest),
                    func: "__result_ok".to_string(),
                    args: vec![val],
                });
                Value::Temp(dest)
            }

            Expr::Err(inner, _) => {
                // Err(value) -> create Result with tag 1
                let val = self.lower_expr(inner);
                let dest = self.func.new_temp();
                self.func.emit(Instruction::Call {
                    dest: Some(dest),
                    func: "__result_err".to_string(),
                    args: vec![val],
                });
                Value::Temp(dest)
            }

            Expr::Some(inner, _) => {
                // Some(value) -> create Option with tag 1
                let val = self.lower_expr(inner);
                let dest = self.func.new_temp();
                self.func.emit(Instruction::Call {
                    dest: Some(dest),
                    func: "__option_some".to_string(),
                    args: vec![val],
                });
                Value::Temp(dest)
            }

            Expr::MethodCall { object, method, args, span: _ } => {
                // Lower the object (receiver)
                let obj_val = self.lower_expr(object);

                // Lower arguments
                let mut arg_values: Vec<_> = args.iter().map(|a| self.lower_expr(a)).collect();

                // Get struct name for method resolution
                // For now, we generate a generic method call
                // In a full implementation, we'd resolve the struct type
                let method_name = match object.as_ref() {
                    Expr::Ident(_name, _) => {
                        // Try to find the variable's type
                        format!("__method_{}", method)
                    }
                    _ => format!("__method_{}", method),
                };

                // Prepend self to arguments
                arg_values.insert(0, obj_val);

                let dest = self.func.new_temp();
                self.func.emit(Instruction::Call {
                    dest: Some(dest),
                    func: method_name,
                    args: arg_values,
                });

                Value::Temp(dest)
            }

            Expr::Match { expr, arms, span: _ } => {
                // Lower the scrutinee (expression being matched)
                let scrutinee = self.lower_expr(expr);

                // Generate labels for each arm and the end
                let end_label = self.new_label("match_end");
                let result_dest = self.func.new_temp();

                // For simple patterns (literals, wildcards), we use conditional branches
                // For complex patterns, we'd need more sophisticated lowering

                let mut arm_labels: Vec<String> = Vec::new();
                for i in 0..arms.len() {
                    arm_labels.push(self.new_label(&format!("match_arm{}", i)));
                }
                arm_labels.push(end_label.clone()); // Fallback to end if no match

                // For each arm, generate comparison and branch
                for (i, arm) in arms.iter().enumerate() {
                    let arm_label = arm_labels[i].clone();
                    let next_label = arm_labels.get(i + 1).cloned().unwrap_or_else(|| end_label.clone());

                    // Generate pattern match check
                    let matched = self.lower_pattern_check(&arm.pattern, &scrutinee);

                    // Handle guard
                    let final_cond = if let Some(guard) = &arm.guard {
                        let guard_val = self.lower_expr(guard);
                        // matched AND guard
                        let temp = self.func.new_temp();
                        self.func.emit(Instruction::Binary {
                            dest: temp,
                            op: BinaryOp::And,
                            left: matched,
                            right: guard_val,
                        });
                        Value::Temp(temp)
                    } else {
                        matched
                    };

                    self.func.emit(Instruction::CondBranch {
                        cond: final_cond,
                        then_label: arm_label.clone(),
                        else_label: next_label.clone(),
                    });

                    // Arm body block
                    self.func.new_block(&arm_label);

                    // Bind pattern variables
                    self.bind_pattern_vars(&arm.pattern, &scrutinee);

                    // Lower arm body
                    let mut arm_result = Value::Void;
                    for stmt in &arm.body {
                        match stmt {
                            Stmt::Return { value, .. } => {
                                if let Some(v) = value {
                                    arm_result = self.lower_expr(v);
                                }
                            }
                            Stmt::Expr(e) => {
                                arm_result = self.lower_expr(e);
                            }
                            _ => {
                                self.lower_stmt(stmt);
                            }
                        }
                    }

                    // Store result in phi destination
                    self.func.emit(Instruction::Store {
                        value: arm_result,
                        ptr: Value::Temp(result_dest),
                    });

                    if !self.func.current_block().is_terminated() {
                        self.func.emit(Instruction::Branch { target: end_label.clone() });
                    }
                }

                // End block
                self.func.new_block(&end_label);
                Value::Temp(result_dest)
            }

            Expr::Try { expr, span: _ } => {
                // Lower the inner expression
                let inner_val = self.lower_expr(expr);

                // Generate labels for success and error paths
                let ok_label = self.new_label("try_ok");
                let err_label = self.new_label("try_err");
                let end_label = self.new_label("try_end");

                // Check if the result is Ok/Some
                let is_ok = self.func.new_temp();
                self.func.emit(Instruction::Call {
                    dest: Some(is_ok),
                    func: "__try_is_ok".to_string(),
                    args: vec![inner_val.clone()],
                });

                self.func.emit(Instruction::CondBranch {
                    cond: Value::Temp(is_ok),
                    then_label: ok_label.clone(),
                    else_label: err_label.clone(),
                });

                // Error path - early return
                self.func.new_block(&err_label);
                let err_val = self.func.new_temp();
                self.func.emit(Instruction::Call {
                    dest: Some(err_val),
                    func: "__try_propagate".to_string(),
                    args: vec![inner_val.clone()],
                });
                self.func.emit(Instruction::Return(Value::Temp(err_val)));

                // Ok path - unwrap and continue
                self.func.new_block(&ok_label);
                let result = self.func.new_temp();
                self.func.emit(Instruction::Call {
                    dest: Some(result),
                    func: "__try_unwrap".to_string(),
                    args: vec![inner_val],
                });
                self.func.emit(Instruction::Branch { target: end_label.clone() });

                // End block
                self.func.new_block(&end_label);
                Value::Temp(result)
            }

            Expr::StringInterpolation { parts, span: _ } => {
                // String interpolation is lowered as a format! call
                let dest = self.func.new_temp();

                // Collect all the parts and expressions
                let mut format_parts: Vec<Value> = Vec::new();

                for part in parts {
                    match part {
                        StringPart::Literal(s) => {
                            let idx = self.ctx.module.add_string(s.clone());
                            format_parts.push(Value::ConstString(idx));
                        }
                        StringPart::Expr(expr) => {
                            let val = self.lower_expr(expr);
                            format_parts.push(val);
                        }
                    }
                }

                // Call the string formatting function
                self.func.emit(Instruction::Call {
                    dest: Some(dest),
                    func: "__string_format".to_string(),
                    args: format_parts,
                });

                Value::Temp(dest)
            }

            Expr::Closure { params, return_type: _, body, span: _ } => {
                // For closures, we generate a unique name and create a separate function
                // Then return a reference to that function

                // Generate unique closure name
                let closure_id = self.ctx.label_counter;
                self.ctx.label_counter += 1;
                let closure_name = format!("__closure_{}", closure_id);

                // Create closure function
                let mut closure_func = Function::new(&closure_name, IrType::I64, false);

                // Add parameters
                for param in params {
                    let ty = param.ty.as_ref()
                        .map(IrType::from_mendes_type)
                        .unwrap_or(IrType::I64);
                    closure_func.add_param(&param.name, ty);
                }

                // Lower the body
                // We need a new lowerer for the closure
                let mut closure_lowerer = FunctionLowerer::new(self.ctx, &mut closure_func);

                match body {
                    ClosureBody::Expr(expr) => {
                        let result = closure_lowerer.lower_expr(expr);
                        closure_lowerer.func.emit(Instruction::Return(result));
                    }
                    ClosureBody::Block(stmts) => {
                        for stmt in stmts {
                            closure_lowerer.lower_stmt(stmt);
                        }
                        if !closure_lowerer.func.current_block().is_terminated() {
                            closure_lowerer.func.emit(Instruction::Return(Value::Void));
                        }
                    }
                }

                // Add closure function to module
                self.ctx.module.add_function(closure_func);

                // Return reference to closure function
                Value::Global(closure_name)
            }

            Expr::Tuple { elements, span: _ } => {
                // Lower tuple as a struct with fields named field0, field1, etc.
                let tuple_type = format!("tuple_{}", elements.len());
                let dest = self.func.new_temp();
                let dest_name = format!("__t{}", dest);

                // Lower each element
                let element_values: Vec<Value> = elements.iter().map(|e| self.lower_expr(e)).collect();

                // Create a struct allocation for the tuple
                self.func.emit(Instruction::Alloca {
                    dest: dest_name.clone(),
                    ty: IrType::Struct(tuple_type.clone()),
                });

                // Set each field
                for (i, val) in element_values.into_iter().enumerate() {
                    self.func.emit(Instruction::SetField {
                        ptr: Value::Temp(dest),
                        struct_name: tuple_type.clone(),
                        field_index: i,
                        field_name: format!("field{}", i),
                        value: val,
                    });
                }

                Value::Temp(dest)
            }

            Expr::Range { start, end, inclusive, span: _ } => {
                // Lower range as a struct with start, end, inclusive fields
                let dest = self.func.new_temp();
                let dest_name = format!("__t{}", dest);

                let start_val = start.as_ref().map(|e| self.lower_expr(e));
                let end_val = end.as_ref().map(|e| self.lower_expr(e));

                // Create range struct
                self.func.emit(Instruction::Alloca {
                    dest: dest_name.clone(),
                    ty: IrType::Struct("Range".to_string()),
                });

                // Set start field (0 if not provided)
                self.func.emit(Instruction::SetField {
                    ptr: Value::Temp(dest),
                    struct_name: "Range".to_string(),
                    field_index: 0,
                    field_name: "start".to_string(),
                    value: start_val.unwrap_or(Value::ConstInt(0)),
                });

                // Set end field (max if not provided)
                self.func.emit(Instruction::SetField {
                    ptr: Value::Temp(dest),
                    struct_name: "Range".to_string(),
                    field_index: 1,
                    field_name: "end".to_string(),
                    value: end_val.unwrap_or(Value::ConstInt(i64::MAX)),
                });

                // Set inclusive flag
                self.func.emit(Instruction::SetField {
                    ptr: Value::Temp(dest),
                    struct_name: "Range".to_string(),
                    field_index: 2,
                    field_name: "inclusive".to_string(),
                    value: Value::ConstBool(*inclusive),
                });

                Value::Temp(dest)
            }
        }
    }

    /// Lower pattern check - returns a boolean Value indicating if pattern matches
    fn lower_pattern_check(&mut self, pattern: &Pattern, scrutinee: &Value) -> Value {
        match pattern {
            Pattern::Wildcard(_) => {
                // Wildcard always matches
                Value::ConstBool(true)
            }

            Pattern::Literal(expr) => {
                // Compare scrutinee with literal
                let lit_val = self.lower_expr(expr);
                let dest = self.func.new_temp();
                self.func.emit(Instruction::Compare {
                    dest,
                    op: CompareOp::Eq,
                    left: scrutinee.clone(),
                    right: lit_val,
                });
                Value::Temp(dest)
            }

            Pattern::Ident { .. } => {
                // Identifier pattern always matches (it binds)
                Value::ConstBool(true)
            }

            Pattern::Variant { enum_name: _, variant, data, span: _ } => {
                // For Option/Result, we check the tag
                let tag_check = match variant.as_str() {
                    "None" => {
                        let dest = self.func.new_temp();
                        self.func.emit(Instruction::Call {
                            dest: Some(dest),
                            func: "__is_none".to_string(),
                            args: vec![scrutinee.clone()],
                        });
                        Value::Temp(dest)
                    }
                    "Some" => {
                        let dest = self.func.new_temp();
                        self.func.emit(Instruction::Call {
                            dest: Some(dest),
                            func: "__is_some".to_string(),
                            args: vec![scrutinee.clone()],
                        });
                        Value::Temp(dest)
                    }
                    "Ok" => {
                        let dest = self.func.new_temp();
                        self.func.emit(Instruction::Call {
                            dest: Some(dest),
                            func: "__is_ok".to_string(),
                            args: vec![scrutinee.clone()],
                        });
                        Value::Temp(dest)
                    }
                    "Err" => {
                        let dest = self.func.new_temp();
                        self.func.emit(Instruction::Call {
                            dest: Some(dest),
                            func: "__is_err".to_string(),
                            args: vec![scrutinee.clone()],
                        });
                        Value::Temp(dest)
                    }
                    _ => {
                        // User-defined enum variant - check tag
                        let dest = self.func.new_temp();
                        self.func.emit(Instruction::Call {
                            dest: Some(dest),
                            func: format!("__is_variant_{}", variant),
                            args: vec![scrutinee.clone()],
                        });
                        Value::Temp(dest)
                    }
                };

                // For tuple/struct variants, also check nested patterns
                match data {
                    VariantPatternData::Unit => tag_check,
                    VariantPatternData::Tuple(patterns) => {
                        if patterns.is_empty() {
                            tag_check
                        } else {
                            // For simplicity, just return tag_check
                            // Full implementation would extract inner values and check them
                            tag_check
                        }
                    }
                    VariantPatternData::Struct(fields) => {
                        let patterns: Vec<_> = fields.iter().filter_map(|(_, p)| p.clone()).collect();
                        if patterns.is_empty() {
                            tag_check
                        } else {
                            // For simplicity, just return tag_check
                            // Full implementation would extract inner values and check them
                            tag_check
                        }
                    }
                }
            }

            Pattern::Or(patterns, _) => {
                // OR pattern: check each alternative
                if patterns.is_empty() {
                    return Value::ConstBool(false);
                }

                let mut result = self.lower_pattern_check(&patterns[0], scrutinee);
                for pattern in patterns.iter().skip(1) {
                    let check = self.lower_pattern_check(pattern, scrutinee);
                    let dest = self.func.new_temp();
                    self.func.emit(Instruction::Binary {
                        dest,
                        op: BinaryOp::Or,
                        left: result,
                        right: check,
                    });
                    result = Value::Temp(dest);
                }
                result
            }

            Pattern::Tuple(patterns, _) => {
                // For tuple patterns, check each element
                if patterns.is_empty() {
                    return Value::ConstBool(true);
                }

                let mut result = Value::ConstBool(true);
                for (i, pattern) in patterns.iter().enumerate() {
                    // Get tuple element
                    let elem_dest = self.func.new_temp();
                    self.func.emit(Instruction::GetElement {
                        dest: elem_dest,
                        ptr: scrutinee.clone(),
                        index: Value::ConstInt(i as i64),
                    });

                    let elem_check = self.lower_pattern_check(pattern, &Value::Temp(elem_dest));
                    let dest = self.func.new_temp();
                    self.func.emit(Instruction::Binary {
                        dest,
                        op: BinaryOp::And,
                        left: result,
                        right: elem_check,
                    });
                    result = Value::Temp(dest);
                }
                result
            }

            Pattern::Struct { name: _, fields, span: _ } => {
                // For struct patterns, check each field
                if fields.is_empty() {
                    return Value::ConstBool(true);
                }

                let mut result = Value::ConstBool(true);
                for (i, (_, pattern)) in fields.iter().enumerate() {
                    if let Some(pat) = pattern {
                        // Get field value
                        let field_dest = self.func.new_temp();
                        self.func.emit(Instruction::GetElement {
                            dest: field_dest,
                            ptr: scrutinee.clone(),
                            index: Value::ConstInt(i as i64),
                        });

                        let field_check = self.lower_pattern_check(pat, &Value::Temp(field_dest));
                        let dest = self.func.new_temp();
                        self.func.emit(Instruction::Binary {
                            dest,
                            op: BinaryOp::And,
                            left: result,
                            right: field_check,
                        });
                        result = Value::Temp(dest);
                    }
                }
                result
            }

            Pattern::Range { start, end, inclusive, span: _ } => {
                // Range pattern: start <= scrutinee < end (or <= for inclusive)
                let mut result = Value::ConstBool(true);

                if let Some(s) = start {
                    let start_val = self.lower_expr(s);
                    let dest = self.func.new_temp();
                    self.func.emit(Instruction::Compare {
                        dest,
                        op: CompareOp::Ge,
                        left: scrutinee.clone(),
                        right: start_val,
                    });
                    let and_dest = self.func.new_temp();
                    self.func.emit(Instruction::Binary {
                        dest: and_dest,
                        op: BinaryOp::And,
                        left: result,
                        right: Value::Temp(dest),
                    });
                    result = Value::Temp(and_dest);
                }

                if let Some(e) = end {
                    let end_val = self.lower_expr(e);
                    let dest = self.func.new_temp();
                    self.func.emit(Instruction::Compare {
                        dest,
                        op: if *inclusive { CompareOp::Le } else { CompareOp::Lt },
                        left: scrutinee.clone(),
                        right: end_val,
                    });
                    let and_dest = self.func.new_temp();
                    self.func.emit(Instruction::Binary {
                        dest: and_dest,
                        op: BinaryOp::And,
                        left: result,
                        right: Value::Temp(dest),
                    });
                    result = Value::Temp(and_dest);
                }

                result
            }
        }
    }

    /// Bind pattern variables (extract values and store in locals)
    fn bind_pattern_vars(&mut self, pattern: &Pattern, scrutinee: &Value) {
        match pattern {
            Pattern::Ident { name, mutable: _, span: _ } => {
                // Bind the scrutinee to the variable
                self.func.add_local(name.clone(), IrType::I64); // Simplified type
                self.func.emit(Instruction::Alloca {
                    dest: name.clone(),
                    ty: IrType::I64,
                });
                self.func.emit(Instruction::Store {
                    value: scrutinee.clone(),
                    ptr: Value::Local(name.clone()),
                });
                self.vars.insert(name.clone(), scrutinee.clone());
            }

            Pattern::Variant { data, .. } => {
                // Extract inner value and bind nested patterns
                match data {
                    VariantPatternData::Tuple(patterns) => {
                        for (i, pat) in patterns.iter().enumerate() {
                            let inner_dest = self.func.new_temp();
                            self.func.emit(Instruction::Call {
                                dest: Some(inner_dest),
                                func: "__extract_variant_field".to_string(),
                                args: vec![scrutinee.clone(), Value::ConstInt(i as i64)],
                            });
                            self.bind_pattern_vars(pat, &Value::Temp(inner_dest));
                        }
                    }
                    VariantPatternData::Struct(fields) => {
                        for (i, (name, pat)) in fields.iter().enumerate() {
                            let inner_dest = self.func.new_temp();
                            self.func.emit(Instruction::Call {
                                dest: Some(inner_dest),
                                func: "__extract_variant_field".to_string(),
                                args: vec![scrutinee.clone(), Value::ConstInt(i as i64)],
                            });
                            if let Some(p) = pat {
                                self.bind_pattern_vars(p, &Value::Temp(inner_dest));
                            } else {
                                // Shorthand: bind field name directly
                                self.func.add_local(name.clone(), IrType::I64);
                                self.func.emit(Instruction::Alloca {
                                    dest: name.clone(),
                                    ty: IrType::I64,
                                });
                                self.func.emit(Instruction::Store {
                                    value: Value::Temp(inner_dest),
                                    ptr: Value::Local(name.clone()),
                                });
                                self.vars.insert(name.clone(), Value::Temp(inner_dest));
                            }
                        }
                    }
                    VariantPatternData::Unit => {}
                }
            }

            Pattern::Tuple(patterns, _) => {
                for (i, pat) in patterns.iter().enumerate() {
                    let elem_dest = self.func.new_temp();
                    self.func.emit(Instruction::GetElement {
                        dest: elem_dest,
                        ptr: scrutinee.clone(),
                        index: Value::ConstInt(i as i64),
                    });
                    self.bind_pattern_vars(pat, &Value::Temp(elem_dest));
                }
            }

            Pattern::Struct { fields, .. } => {
                for (i, (name, pat)) in fields.iter().enumerate() {
                    let field_dest = self.func.new_temp();
                    self.func.emit(Instruction::GetElement {
                        dest: field_dest,
                        ptr: scrutinee.clone(),
                        index: Value::ConstInt(i as i64),
                    });
                    if let Some(p) = pat {
                        self.bind_pattern_vars(p, &Value::Temp(field_dest));
                    } else {
                        // Shorthand: bind field name directly
                        self.func.add_local(name.clone(), IrType::I64);
                        self.func.emit(Instruction::Alloca {
                            dest: name.clone(),
                            ty: IrType::I64,
                        });
                        self.func.emit(Instruction::Store {
                            value: Value::Temp(field_dest),
                            ptr: Value::Local(name.clone()),
                        });
                        self.vars.insert(name.clone(), Value::Temp(field_dest));
                    }
                }
            }

            Pattern::Or(patterns, _) => {
                // For OR patterns, bind from first pattern (they should all bind same vars)
                if let Some(first) = patterns.first() {
                    self.bind_pattern_vars(first, scrutinee);
                }
            }

            // Patterns that don't bind variables
            Pattern::Wildcard(_) | Pattern::Literal(_) | Pattern::Range { .. } => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mendes_lexer::Lexer;
    use mendes_parser::parse;

    fn lower_source(source: &str) -> Module {
        let mut lexer = Lexer::new(source, 0);
        let tokens = lexer.tokenize();
        let (program, _) = parse(tokens);
        lower_program(&program)
    }

    #[test]
    fn test_lower_simple_function() {
        let source = r#"fn add(a: int, b: int) -> int:
    return a + b
"#;
        let module = lower_source(source);

        assert_eq!(module.functions.len(), 1);
        assert_eq!(module.functions[0].name, "add");
        assert_eq!(module.functions[0].params.len(), 2);
    }

    #[test]
    fn test_lower_server() {
        let source = r#"server:
    host "0.0.0.0"
    port 8080
"#;
        let module = lower_source(source);

        assert!(module.server.is_some());
        let server = module.server.unwrap();
        assert_eq!(server.host, "0.0.0.0");
        assert_eq!(server.port, 8080);
    }

    #[test]
    fn test_lower_api() {
        let source = r#"api GET /health:
    return string
    return "ok"
"#;
        let module = lower_source(source);

        assert_eq!(module.routes.len(), 1);
        assert_eq!(module.routes[0].method, "GET");
        assert_eq!(module.routes[0].path, "/health");
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_lower_struct() {
        let source = r#"struct User:
    id: int
    name: string
"#;
        let module = lower_source(source);

        assert!(module.get_struct("User").is_some());
        let user = module.get_struct("User").unwrap();
        assert_eq!(user.fields.len(), 2);
    }
}
