//! IR Module - high-level structure
//!
//! Contains module definition, functions, basic blocks and HTTP routes.

use crate::types::{IrType, StructDef, GenericParam};
use crate::instruction::{Instruction, Value};
use std::collections::HashMap;
use std::fmt;

/// IR Module - represents a complete program
#[derive(Debug)]
pub struct Module {
    /// Module name
    pub name: String,
    /// Functions
    pub functions: Vec<Function>,
    /// HTTP routes
    pub routes: Vec<HttpRoute>,
    /// WebSocket routes
    pub websocket_routes: Vec<WsRoute>,
    /// Server configuration
    pub server: Option<ServerConfig>,
    /// Defined structs
    pub structs: HashMap<String, StructDef>,
    /// Trait definitions
    pub traits: HashMap<String, TraitDef>,
    /// Trait implementations
    pub impls: Vec<ImplDef>,
    /// Type aliases
    pub type_aliases: HashMap<String, TypeAlias>,
    /// Globals
    pub globals: Vec<Global>,
    /// Constant string table
    pub string_table: Vec<String>,
    /// Database connections
    pub databases: Vec<DatabaseConfig>,
    /// Middlewares
    pub middlewares: Vec<String>,
}

impl Module {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            functions: Vec::new(),
            routes: Vec::new(),
            websocket_routes: Vec::new(),
            server: None,
            structs: HashMap::new(),
            traits: HashMap::new(),
            impls: Vec::new(),
            type_aliases: HashMap::new(),
            globals: Vec::new(),
            string_table: Vec::new(),
            databases: Vec::new(),
            middlewares: Vec::new(),
        }
    }

    /// Adds a string to the table and returns its index
    pub fn add_string(&mut self, s: String) -> usize {
        if let Some(idx) = self.string_table.iter().position(|x| x == &s) {
            idx
        } else {
            let idx = self.string_table.len();
            self.string_table.push(s);
            idx
        }
    }

    /// Adds a function
    pub fn add_function(&mut self, func: Function) {
        self.functions.push(func);
    }

    /// Adds an HTTP route
    pub fn add_route(&mut self, route: HttpRoute) {
        self.routes.push(route);
    }

    /// Adds a WebSocket route
    pub fn add_websocket_route(&mut self, route: WsRoute) {
        self.websocket_routes.push(route);
    }

    /// Adds a struct
    pub fn add_struct(&mut self, def: StructDef) {
        self.structs.insert(def.name.clone(), def);
    }

    /// Finds a struct by name
    pub fn get_struct(&self, name: &str) -> Option<&StructDef> {
        self.structs.get(name)
    }

    /// Adds a trait definition
    pub fn add_trait(&mut self, def: TraitDef) {
        self.traits.insert(def.name.clone(), def);
    }

    /// Finds a trait by name
    pub fn get_trait(&self, name: &str) -> Option<&TraitDef> {
        self.traits.get(name)
    }

    /// Adds a trait implementation
    pub fn add_impl(&mut self, impl_def: ImplDef) {
        self.impls.push(impl_def);
    }

    /// Adds a type alias
    pub fn add_type_alias(&mut self, alias: TypeAlias) {
        self.type_aliases.insert(alias.name.clone(), alias);
    }

    /// Finds a type alias by name
    pub fn get_type_alias(&self, name: &str) -> Option<&TypeAlias> {
        self.type_aliases.get(name)
    }

    /// Finds a function by name
    pub fn get_function(&self, name: &str) -> Option<&Function> {
        self.functions.iter().find(|f| f.name == name)
    }

    /// Finds a mutable function by name
    pub fn get_function_mut(&mut self, name: &str) -> Option<&mut Function> {
        self.functions.iter_mut().find(|f| f.name == name)
    }
}

impl fmt::Display for Module {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "; Module: {}", self.name)?;
        writeln!(f)?;

        // Strings
        if !self.string_table.is_empty() {
            writeln!(f, "; String table")?;
            for (i, s) in self.string_table.iter().enumerate() {
                writeln!(f, "@str{} = \"{}\"", i, s.escape_default())?;
            }
            writeln!(f)?;
        }

        // Structs
        for (name, def) in &self.structs {
            writeln!(f, "; Struct {}", name)?;
            write!(f, "%{} = type {{ ", name)?;
            for (i, (fname, fty)) in def.fields.iter().enumerate() {
                if i > 0 { write!(f, ", ")?; }
                write!(f, "{} {}", fty, fname)?;
            }
            writeln!(f, " }}")?;
        }
        if !self.structs.is_empty() {
            writeln!(f)?;
        }

        // Server config
        if let Some(server) = &self.server {
            writeln!(f, "; Server: {}:{}", server.host, server.port)?;
            writeln!(f)?;
        }

        // Routes
        if !self.routes.is_empty() {
            writeln!(f, "; HTTP Routes")?;
            for route in &self.routes {
                writeln!(f, ";   {} {} -> @{}", route.method, route.path, route.handler)?;
            }
            writeln!(f)?;
        }

        // Functions
        for func in &self.functions {
            writeln!(f, "{}", func)?;
        }

        Ok(())
    }
}

/// Function in IR
#[derive(Debug)]
pub struct Function {
    /// Function name
    pub name: String,
    /// Generic type parameters
    pub generic_params: Vec<GenericParam>,
    /// Parameters (name, type)
    pub params: Vec<(String, IrType)>,
    /// Return type
    pub return_type: IrType,
    /// Whether it is async
    pub is_async: bool,
    /// Basic blocks
    pub blocks: Vec<BasicBlock>,
    /// Allocated local variables
    pub locals: HashMap<String, IrType>,
    /// Next temporary ID
    next_temp: u32,
}

impl Function {
    pub fn new(name: impl Into<String>, return_type: IrType, is_async: bool) -> Self {
        let mut func = Self {
            name: name.into(),
            generic_params: Vec::new(),
            params: Vec::new(),
            return_type,
            is_async,
            blocks: Vec::new(),
            locals: HashMap::new(),
            next_temp: 0,
        };
        // Create entry block
        func.blocks.push(BasicBlock::new("entry"));
        func
    }

    /// Sets generic parameters
    pub fn set_generic_params(&mut self, params: Vec<GenericParam>) {
        self.generic_params = params;
    }

    /// Adds a generic parameter
    pub fn add_generic_param(&mut self, param: GenericParam) {
        self.generic_params.push(param);
    }

    /// Adds parameter
    pub fn add_param(&mut self, name: impl Into<String>, ty: IrType) {
        self.params.push((name.into(), ty));
    }

    /// Creates a new temporary
    pub fn new_temp(&mut self) -> u32 {
        let id = self.next_temp;
        self.next_temp += 1;
        id
    }

    /// Adds local variable
    pub fn add_local(&mut self, name: impl Into<String>, ty: IrType) {
        self.locals.insert(name.into(), ty);
    }

    /// Finds block by label
    pub fn get_block(&self, label: &str) -> Option<&BasicBlock> {
        self.blocks.iter().find(|b| b.label == label)
    }

    /// Finds mutable block by label
    pub fn get_block_mut(&mut self, label: &str) -> Option<&mut BasicBlock> {
        self.blocks.iter_mut().find(|b| b.label == label)
    }

    /// Returns the current block (last)
    pub fn current_block(&self) -> &BasicBlock {
        self.blocks.last().expect("Function must have at least one block")
    }

    /// Returns the current mutable block
    pub fn current_block_mut(&mut self) -> &mut BasicBlock {
        self.blocks.last_mut().expect("Function must have at least one block")
    }

    /// Creates a new block
    pub fn new_block(&mut self, label: impl Into<String>) -> &mut BasicBlock {
        self.blocks.push(BasicBlock::new(label));
        self.blocks.last_mut().unwrap()
    }

    /// Adds instruction to current block
    pub fn emit(&mut self, inst: Instruction) {
        self.current_block_mut().instructions.push(inst);
    }

    /// Emits instruction and returns the destination temporary
    pub fn emit_with_dest(&mut self, inst: Instruction) -> Value {
        let dest = inst.dest().expect("Instruction must have destination");
        self.current_block_mut().instructions.push(inst);
        Value::Temp(dest)
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Signature
        let async_str = if self.is_async { "async " } else { "" };
        write!(f, "define {}{}@{}(", async_str, self.return_type, self.name)?;
        for (i, (name, ty)) in self.params.iter().enumerate() {
            if i > 0 { write!(f, ", ")?; }
            write!(f, "{} %{}", ty, name)?;
        }
        writeln!(f, ") {{")?;

        // Locals
        for (name, ty) in &self.locals {
            writeln!(f, "  %{} = alloca {}", name, ty)?;
        }
        if !self.locals.is_empty() {
            writeln!(f)?;
        }

        // Blocks
        for block in &self.blocks {
            writeln!(f, "{}:", block.label)?;
            for inst in &block.instructions {
                writeln!(f, "  {}", inst)?;
            }
        }

        writeln!(f, "}}")
    }
}

/// Basic Block - sequence of instructions without branches
#[derive(Debug)]
pub struct BasicBlock {
    /// Block label
    pub label: String,
    /// Instructions
    pub instructions: Vec<Instruction>,
}

impl BasicBlock {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            instructions: Vec::new(),
        }
    }

    /// Checks if the block is terminated (has a terminator instruction)
    pub fn is_terminated(&self) -> bool {
        self.instructions.last().map(|i| i.is_terminator()).unwrap_or(false)
    }

    /// Adds instruction
    pub fn push(&mut self, inst: Instruction) {
        self.instructions.push(inst);
    }
}

/// HTTP Route
#[derive(Debug, Clone)]
pub struct HttpRoute {
    /// HTTP method (GET, POST, etc)
    pub method: String,
    /// Route path (/users/{id})
    pub path: String,
    /// Handler function name
    pub handler: String,
    /// Middlewares to apply
    pub middlewares: Vec<String>,
    /// Whether it is async
    pub is_async: bool,
}

impl HttpRoute {
    pub fn new(method: impl Into<String>, path: impl Into<String>, handler: impl Into<String>) -> Self {
        Self {
            method: method.into(),
            path: path.into(),
            handler: handler.into(),
            middlewares: Vec::new(),
            is_async: false,
        }
    }
}

/// WebSocket route
#[derive(Debug, Clone)]
pub struct WsRoute {
    /// Route path (/chat, /ws/{room})
    pub path: String,
    /// Handler function name for on_connect
    pub on_connect: Option<String>,
    /// Handler function name for on_message
    pub on_message: Option<String>,
    /// Handler function name for on_disconnect
    pub on_disconnect: Option<String>,
    /// Middlewares to apply
    pub middlewares: Vec<String>,
}

impl WsRoute {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            on_connect: None,
            on_message: None,
            on_disconnect: None,
            middlewares: Vec::new(),
        }
    }
}

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

/// Database configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub name: String,
    pub db_type: String, // postgres, mysql, sqlite
    pub url: String,
    pub pool_size: u32,
}

/// Global (global variable or constant)
#[derive(Debug, Clone)]
pub struct Global {
    pub name: String,
    pub ty: IrType,
    pub initializer: Option<Value>,
    pub is_const: bool,
}

impl Global {
    pub fn new(name: impl Into<String>, ty: IrType) -> Self {
        Self {
            name: name.into(),
            ty,
            initializer: None,
            is_const: false,
        }
    }

    pub fn with_init(mut self, value: Value) -> Self {
        self.initializer = Some(value);
        self
    }

    pub fn constant(mut self) -> Self {
        self.is_const = true;
        self
    }
}

/// Trait definition in IR
#[derive(Debug, Clone)]
pub struct TraitDef {
    /// Trait name
    pub name: String,
    /// Generic type parameters
    pub generic_params: Vec<GenericParam>,
    /// Method signatures (name, params, return_type, is_async)
    pub methods: Vec<TraitMethodDef>,
}

impl TraitDef {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            generic_params: Vec::new(),
            methods: Vec::new(),
        }
    }

    pub fn add_generic_param(&mut self, param: GenericParam) {
        self.generic_params.push(param);
    }

    pub fn add_method(&mut self, method: TraitMethodDef) {
        self.methods.push(method);
    }
}

/// Trait method definition (signature only)
#[derive(Debug, Clone)]
pub struct TraitMethodDef {
    /// Method name
    pub name: String,
    /// Parameters (name, type)
    pub params: Vec<(String, IrType)>,
    /// Return type
    pub return_type: IrType,
    /// Whether it is async
    pub is_async: bool,
    /// Receiver type: 0 = &self, 1 = &mut self, 2 = self
    pub receiver: u8,
}

impl TraitMethodDef {
    pub fn new(name: impl Into<String>, return_type: IrType) -> Self {
        Self {
            name: name.into(),
            params: Vec::new(),
            return_type,
            is_async: false,
            receiver: 0, // &self by default
        }
    }

    pub fn add_param(&mut self, name: impl Into<String>, ty: IrType) {
        self.params.push((name.into(), ty));
    }

    pub fn set_async(mut self, is_async: bool) -> Self {
        self.is_async = is_async;
        self
    }

    pub fn set_receiver(mut self, receiver: u8) -> Self {
        self.receiver = receiver;
        self
    }
}

/// Trait implementation
#[derive(Debug, Clone)]
pub struct ImplDef {
    /// Trait being implemented
    pub trait_name: String,
    /// Type implementing the trait
    pub type_name: String,
    /// Generic type parameters
    pub generic_params: Vec<GenericParam>,
    /// Implemented method names (functions are stored separately)
    pub methods: Vec<String>,
}

impl ImplDef {
    pub fn new(trait_name: impl Into<String>, type_name: impl Into<String>) -> Self {
        Self {
            trait_name: trait_name.into(),
            type_name: type_name.into(),
            generic_params: Vec::new(),
            methods: Vec::new(),
        }
    }

    pub fn add_method(&mut self, name: impl Into<String>) {
        self.methods.push(name.into());
    }
}

/// Type alias
#[derive(Debug, Clone)]
pub struct TypeAlias {
    /// Alias name
    pub name: String,
    /// Target type
    pub target: IrType,
}

impl TypeAlias {
    pub fn new(name: impl Into<String>, target: IrType) -> Self {
        Self {
            name: name.into(),
            target,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instruction::BinaryOp;

    #[test]
    fn test_function_creation() {
        let mut func = Function::new("add", IrType::I64, false);
        func.add_param("a", IrType::I64);
        func.add_param("b", IrType::I64);

        let t0 = func.new_temp();
        func.emit(Instruction::Binary {
            dest: t0,
            op: BinaryOp::Add,
            left: Value::Param(0),
            right: Value::Param(1),
        });
        func.emit(Instruction::Return(Value::Temp(t0)));

        assert_eq!(func.params.len(), 2);
        assert_eq!(func.blocks.len(), 1);
        assert_eq!(func.blocks[0].instructions.len(), 2);
    }

    #[test]
    fn test_module_display() {
        let mut module = Module::new("test");
        module.add_string("Hello, World!".to_string());

        let mut func = Function::new("main", IrType::Void, false);
        func.emit(Instruction::Return(Value::Void));
        module.add_function(func);

        let output = module.to_string();
        assert!(output.contains("; Module: test"));
        assert!(output.contains("define void@main"));
    }
}
