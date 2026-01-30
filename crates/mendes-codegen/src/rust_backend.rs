//! Rust Backend - Generates Rust code that uses mendes-runtime
//!
//! This backend generates Rust code that can be compiled
//! together with mendes-runtime to create native executables.

use crate::CodeGen;
use mendes_ir::{Module, Function, Instruction, Value, BinaryOp, CompareOp, IrType, GenericParam};
use std::fmt::Write;

/// Information about a detected for loop pattern
struct ForLoopInfo {
    loop_id: String,
    loop_var: String,
    end_val: Value,
    cmp_op: CompareOp,
    body_label: String,
    inc_label: String,
    end_label: String,
}

/// Information about a detected while loop pattern
struct WhileLoopInfo {
    loop_id: String,
    body_label: String,
    end_label: String,
}

/// Rust code generation backend
#[derive(Debug, Default)]
pub struct RustBackend;

impl RustBackend {
    pub fn new() -> Self {
        Self
    }

    /// Format generic parameters for Rust output: <T, U: Trait>
    fn emit_generic_params(&self, params: &[GenericParam]) -> String {
        if params.is_empty() {
            return String::new();
        }

        let formatted: Vec<String> = params.iter().map(|p| {
            if p.bounds.is_empty() {
                p.name.clone()
            } else {
                format!("{}: {}", p.name, p.bounds.join(" + "))
            }
        }).collect();

        format!("<{}>", formatted.join(", "))
    }

    /// Format generic parameter names only (for type references): <T, U>
    fn emit_generic_args(&self, params: &[GenericParam]) -> String {
        if params.is_empty() {
            return String::new();
        }

        let names: Vec<&str> = params.iter().map(|p| p.name.as_str()).collect();
        format!("<{}>", names.join(", "))
    }

    fn emit_type(&self, ty: &IrType) -> String {
        match ty {
            IrType::Void => "()".to_string(),
            IrType::I64 => "i64".to_string(),
            IrType::F64 => "f64".to_string(),
            IrType::Bool => "bool".to_string(),
            IrType::String => "MendesString".to_string(),
            IrType::Ptr(inner) => format!("&{}", self.emit_type(inner)),
            IrType::Array(elem, _) => format!("MendesArray<{}>", self.emit_type(elem)),
            IrType::Struct(name) => self.emit_struct_type(name),
            IrType::Function { params, ret } => {
                let params_str: Vec<_> = params.iter().map(|p| self.emit_type(p)).collect();
                format!("fn({}) -> {}", params_str.join(", "), self.emit_type(ret))
            }
            IrType::Future(inner) => format!("impl Future<Output = {}>", self.emit_type(inner)),
            IrType::Tuple(elems) => {
                let elem_types: Vec<_> = elems.iter().map(|e| self.emit_type(e)).collect();
                format!("({})", elem_types.join(", "))
            }
            IrType::Range(inner) => format!("std::ops::Range<{}>", self.emit_type(inner)),
        }
    }

    fn emit_struct_type(&self, name: &str) -> String {
        // Handle Result<T, E> and Option<T> encoded types
        if name.starts_with("Result_") {
            // Result_Int_String -> MendesResult<i64, MendesString>
            let parts: Vec<&str> = name[7..].split('_').collect();
            if parts.len() >= 2 {
                let ok_type = self.convert_type_name(parts[0]);
                let err_type = self.convert_type_name(parts[1]);
                return format!("MendesResult<{}, {}>", ok_type, err_type);
            }
        } else if name.starts_with("Option_") {
            // Option_Int -> MendesOption<i64>
            let inner = &name[7..];
            let inner_type = self.convert_type_name(inner);
            return format!("MendesOption<{}>", inner_type);
        }
        // Default: return name as-is
        name.to_string()
    }

    fn convert_type_name(&self, name: &str) -> String {
        match name.to_lowercase().as_str() {
            "int" => "i64".to_string(),
            "float" => "f64".to_string(),
            "bool" => "bool".to_string(),
            "string" => "MendesString".to_string(),
            _ => name.to_string(),
        }
    }

    fn emit_value(&self, value: &Value) -> String {
        match value {
            Value::ConstInt(v) => format!("{}", v),
            Value::ConstFloat(bits) => format!("{:.6}_f64", f64::from_bits(*bits)),
            Value::ConstBool(v) => if *v { "true" } else { "false" }.to_string(),
            Value::ConstString(idx) => format!("__str_{}()", idx),
            Value::Local(name) => name.clone(),
            Value::Param(idx) => format!("__arg{}", idx),
            Value::Global(name) => format!("{}", name),
            Value::Temp(id) => format!("__t{}", id),
            Value::Void => "()".to_string(),
        }
    }

    fn emit_binary_op(&self, op: &BinaryOp) -> &'static str {
        match op {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Mod => "%",
            BinaryOp::And => "&&",
            BinaryOp::Or => "||",
            BinaryOp::Xor => "^",
            BinaryOp::Shl => "<<",
            BinaryOp::Shr => ">>",
        }
    }

    fn emit_compare_op(&self, op: &CompareOp) -> &'static str {
        match op {
            CompareOp::Eq => "==",
            CompareOp::Ne => "!=",
            CompareOp::Lt => "<",
            CompareOp::Le => "<=",
            CompareOp::Gt => ">",
            CompareOp::Ge => ">=",
        }
    }

    fn emit_prelude(&self, module: &Module, output: &mut String) {
        writeln!(output, "// Generated by Mendes Compiler").unwrap();
        writeln!(output, "// Do not edit manually").unwrap();
        writeln!(output).unwrap();
        writeln!(output, "#![allow(unused_variables, unused_imports, dead_code, non_snake_case)]").unwrap();
        writeln!(output).unwrap();
        writeln!(output, "use mendes_runtime::{{").unwrap();
        writeln!(output, "    Server, Router, Request, Response,").unwrap();
        writeln!(output, "    MendesString, MendesResult, MendesOption,").unwrap();
        writeln!(output, "    tokio,").unwrap();
        writeln!(output, "}};").unwrap();

        // Add database imports if needed
        if !module.databases.is_empty() {
            writeln!(output, "use std::sync::Arc;").unwrap();
            for db in &module.databases {
                match db.db_type.as_str() {
                    "postgres" => writeln!(output, "use mendes_runtime::PostgresPool;").unwrap(),
                    "mysql" => writeln!(output, "use mendes_runtime::MysqlPool;").unwrap(),
                    "sqlite" => writeln!(output, "use mendes_runtime::SqlitePool;").unwrap(),
                    _ => {}
                }
            }
        }
        writeln!(output).unwrap();
    }

    fn emit_string_table(&self, module: &Module, output: &mut String) {
        if module.string_table.is_empty() {
            return;
        }

        // Use lazy_static for string constants
        writeln!(output, "// String constants").unwrap();
        writeln!(output, "macro_rules! mendes_str {{").unwrap();
        writeln!(output, "    ($s:expr) => {{ MendesString::new($s) }};").unwrap();
        writeln!(output, "}}").unwrap();
        writeln!(output).unwrap();

        for (i, s) in module.string_table.iter().enumerate() {
            let escaped = s.replace('\\', "\\\\")
                          .replace('"', "\\\"")
                          .replace('\n', "\\n")
                          .replace('\r', "\\r")
                          .replace('\t', "\\t");
            writeln!(output, "fn __str_{}() -> MendesString {{ mendes_str!(\"{}\") }}", i, escaped).unwrap();
        }
        writeln!(output).unwrap();
    }

    fn emit_structs(&self, module: &Module, output: &mut String) {
        if module.structs.is_empty() {
            return;
        }

        writeln!(output, "// Struct definitions").unwrap();
        for (name, def) in &module.structs {
            let generic_params = self.emit_generic_params(&def.generic_params);
            writeln!(output, "#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]").unwrap();
            writeln!(output, "pub struct {}{} {{", name, generic_params).unwrap();
            for (field_name, field_type) in &def.fields {
                writeln!(output, "    pub {}: {},", field_name, self.emit_type(field_type)).unwrap();
            }
            writeln!(output, "}}").unwrap();
            writeln!(output).unwrap();
        }
    }

    fn emit_traits(&self, module: &Module, output: &mut String) {
        if module.traits.is_empty() {
            return;
        }

        writeln!(output, "// Trait definitions").unwrap();
        for (name, def) in &module.traits {
            let generic_params = self.emit_generic_params(&def.generic_params);
            writeln!(output, "pub trait {}{} {{", name, generic_params).unwrap();

            for method in &def.methods {
                // Receiver
                let receiver = match method.receiver {
                    0 => "&self",
                    1 => "&mut self",
                    2 => "self",
                    _ => "&self",
                };

                // Async
                let async_str = if method.is_async { "async " } else { "" };

                // Parameters
                let params: Vec<String> = method.params.iter()
                    .map(|(name, ty)| format!("{}: {}", name, self.emit_type(ty)))
                    .collect();

                // Return type
                let return_type = self.emit_type(&method.return_type);

                if params.is_empty() {
                    writeln!(output, "    {}fn {}({}) -> {};",
                        async_str, method.name, receiver, return_type).unwrap();
                } else {
                    writeln!(output, "    {}fn {}({}, {}) -> {};",
                        async_str, method.name, receiver, params.join(", "), return_type).unwrap();
                }
            }

            writeln!(output, "}}").unwrap();
            writeln!(output).unwrap();
        }
    }

    fn emit_impls(&self, module: &Module, output: &mut String) {
        if module.impls.is_empty() {
            return;
        }

        writeln!(output, "// Trait implementations").unwrap();
        for impl_def in &module.impls {
            let generic_params = self.emit_generic_params(&impl_def.generic_params);
            let generic_args = self.emit_generic_args(&impl_def.generic_params);

            // Get the trait definition to know method signatures
            let trait_def = module.get_trait(&impl_def.trait_name);

            writeln!(output, "impl{} {} for {}{} {{",
                generic_params, impl_def.trait_name, impl_def.type_name, generic_args).unwrap();

            // For each method, find the corresponding function and emit it inline
            for method_name in &impl_def.methods {
                // Find the function
                if let Some(func) = module.get_function(method_name) {
                    // Get trait method signature if available
                    let trait_method = trait_def.and_then(|t|
                        t.methods.iter().find(|m| method_name.ends_with(&format!("::{}", m.name)))
                    );

                    // Receiver
                    let receiver = trait_method.map(|m| match m.receiver {
                        0 => "&self",
                        1 => "&mut self",
                        2 => "self",
                        _ => "&self",
                    }).unwrap_or("&self");

                    // Extract just the method name (after last ::)
                    let short_name = method_name.rsplit("::").next().unwrap_or(method_name);

                    // Async
                    let async_str = if func.is_async { "async " } else { "" };

                    // Parameters (skip self)
                    let params: Vec<String> = func.params.iter().skip(1)
                        .enumerate()
                        .map(|(i, (_, ty))| format!("__arg{}: {}", i + 1, self.emit_type(ty)))
                        .collect();

                    // Return type
                    let return_type = self.emit_type(&func.return_type);

                    if params.is_empty() {
                        writeln!(output, "    {}fn {}({}) -> {} {{",
                            async_str, short_name, receiver, return_type).unwrap();
                    } else {
                        writeln!(output, "    {}fn {}({}, {}) -> {} {{",
                            async_str, short_name, receiver, params.join(", "), return_type).unwrap();
                    }

                    // Emit function body
                    self.emit_structured_blocks(func, module, output);

                    writeln!(output, "    }}").unwrap();
                }
            }

            writeln!(output, "}}").unwrap();
            writeln!(output).unwrap();
        }
    }

    fn emit_type_aliases(&self, module: &Module, output: &mut String) {
        if module.type_aliases.is_empty() {
            return;
        }

        writeln!(output, "// Type aliases").unwrap();
        for (name, alias) in &module.type_aliases {
            writeln!(output, "pub type {} = {};", name, self.emit_type(&alias.target)).unwrap();
        }
        writeln!(output).unwrap();
    }

    fn emit_function(&self, func: &Function, module: &Module, output: &mut String) {
        let _is_handler = func.name.starts_with("__http_");
        let return_type = self.emit_type(&func.return_type);
        let generic_params = self.emit_generic_params(&func.generic_params);

        // Signature
        if func.is_async {
            write!(output, "async fn {}{}(", func.name, generic_params).unwrap();
        } else {
            write!(output, "fn {}{}(", func.name, generic_params).unwrap();
        }

        // Parameters - use original names for proper reference in function body
        for (i, (name, ty)) in func.params.iter().enumerate() {
            if i > 0 {
                write!(output, ", ").unwrap();
            }
            // Use original name if it doesn't start with __, otherwise use __arg{i}
            if name.starts_with("__") || name.is_empty() {
                write!(output, "__arg{}: {}", i, self.emit_type(ty)).unwrap();
            } else {
                write!(output, "{}: {}", name, self.emit_type(ty)).unwrap();
            }
        }

        writeln!(output, ") -> {} {{", return_type).unwrap();

        // Local variables
        for (name, ty) in &func.locals {
            writeln!(output, "    let mut {}: {};", name, self.emit_type(ty)).unwrap();
        }

        // Emit a newline after locals if we have any
        if !func.locals.is_empty() {
            writeln!(output).unwrap();
        }

        // Emit structured control flow instead of raw blocks
        self.emit_structured_blocks(func, module, output);

        writeln!(output, "}}").unwrap();
        writeln!(output).unwrap();
    }

    /// Emit blocks with proper structured control flow (if/else, for, while)
    fn emit_structured_blocks(&self, func: &Function, module: &Module, output: &mut String) {
        use std::collections::{HashMap, HashSet};

        // Build a map of block labels to their contents
        let mut block_map: HashMap<&str, &mendes_ir::BasicBlock> = HashMap::new();
        for block in &func.blocks {
            block_map.insert(&block.label, block);
        }

        // Identify loop headers (for_cond_*, while_cond_*)
        let mut loop_headers: HashSet<&str> = HashSet::new();
        for block in &func.blocks {
            if block.label.starts_with("for_cond_") || block.label.starts_with("while_cond_") {
                loop_headers.insert(&block.label);
            }
        }

        // Track which blocks we've already emitted
        let mut emitted: HashSet<&str> = HashSet::new();

        // Start with the entry block
        if let Some(entry) = block_map.get("entry") {
            self.emit_block_with_loops(entry, &block_map, &loop_headers, &mut emitted, module, output, 1);
        }
    }

    /// Detect if this is a for loop pattern and extract loop info
    fn detect_for_loop(&self, cond_label: &str, block_map: &std::collections::HashMap<&str, &mendes_ir::BasicBlock>)
        -> Option<ForLoopInfo>
    {
        // Extract loop ID from label like "for_cond_0"
        let loop_id = cond_label.strip_prefix("for_cond_")?;

        let body_label = format!("for_body_{}", loop_id);
        let inc_label = format!("for_inc_{}", loop_id);
        let end_label = format!("for_end_{}", loop_id);

        // Check all blocks exist
        if !block_map.contains_key(body_label.as_str()) ||
           !block_map.contains_key(inc_label.as_str()) ||
           !block_map.contains_key(end_label.as_str()) {
            return None;
        }

        // Find the loop variable by looking at the condition block's Load instruction
        let cond_block = block_map.get(cond_label)?;
        let mut loop_var = None;
        let mut end_val = None;
        let mut cmp_op = None;

        for inst in &cond_block.instructions {
            match inst {
                Instruction::Load { ptr: Value::Local(name), .. } => {
                    loop_var = Some(name.clone());
                }
                Instruction::Compare { op, right, .. } => {
                    cmp_op = Some(*op);
                    end_val = Some(right.clone());
                }
                _ => {}
            }
        }

        Some(ForLoopInfo {
            loop_id: loop_id.to_string(),
            loop_var: loop_var.unwrap_or_default(),
            end_val: end_val.unwrap_or(Value::ConstInt(0)),
            cmp_op: cmp_op.unwrap_or(CompareOp::Lt),
            body_label,
            inc_label,
            end_label,
        })
    }

    /// Detect if this is a while loop pattern
    fn detect_while_loop(&self, cond_label: &str, block_map: &std::collections::HashMap<&str, &mendes_ir::BasicBlock>)
        -> Option<WhileLoopInfo>
    {
        let loop_id = cond_label.strip_prefix("while_cond_")?;

        let body_label = format!("while_body_{}", loop_id);
        let end_label = format!("while_end_{}", loop_id);

        if !block_map.contains_key(body_label.as_str()) ||
           !block_map.contains_key(end_label.as_str()) {
            return None;
        }

        Some(WhileLoopInfo {
            loop_id: loop_id.to_string(),
            body_label,
            end_label,
        })
    }

    fn emit_block_with_loops<'a>(
        &self,
        block: &'a mendes_ir::BasicBlock,
        block_map: &std::collections::HashMap<&str, &'a mendes_ir::BasicBlock>,
        loop_headers: &std::collections::HashSet<&str>,
        emitted: &mut std::collections::HashSet<&'a str>,
        module: &Module,
        output: &mut String,
        depth: usize,
    ) {
        if emitted.contains(block.label.as_str()) {
            return;
        }

        let indent = "    ".repeat(depth);

        // Check for loop patterns
        for inst in &block.instructions {
            if let Instruction::Branch { target } = inst {
                // Check if we're branching to a for loop header
                if let Some(loop_info) = self.detect_for_loop(target, block_map) {
                    // Emit instructions before the branch
                    for inst2 in &block.instructions {
                        if matches!(inst2, Instruction::Branch { .. }) {
                            break;
                        }
                        self.emit_instruction_indented(inst2, module, output, depth);
                    }
                    emitted.insert(&block.label);

                    // Emit for loop
                    self.emit_for_loop(&loop_info, block_map, emitted, module, output, depth);

                    // Continue with the end block
                    if let Some(end_block) = block_map.get(loop_info.end_label.as_str()) {
                        self.emit_block_with_loops(end_block, block_map, loop_headers, emitted, module, output, depth);
                    }
                    return;
                }

                // Check if we're branching to a while loop header
                if let Some(loop_info) = self.detect_while_loop(target, block_map) {
                    for inst2 in &block.instructions {
                        if matches!(inst2, Instruction::Branch { .. }) {
                            break;
                        }
                        self.emit_instruction_indented(inst2, module, output, depth);
                    }
                    emitted.insert(&block.label);

                    self.emit_while_loop(&loop_info, block_map, emitted, module, output, depth);

                    if let Some(end_block) = block_map.get(loop_info.end_label.as_str()) {
                        self.emit_block_with_loops(end_block, block_map, loop_headers, emitted, module, output, depth);
                    }
                    return;
                }
            }
        }

        // Not a loop pattern, emit normally
        emitted.insert(&block.label);

        // Emit regular instructions
        for inst in &block.instructions {
            match inst {
                Instruction::Branch { target } => {
                    // Follow unconditional branches (but not loop back-edges)
                    if !emitted.contains(target.as_str()) && !loop_headers.contains(target.as_str()) {
                        if let Some(next_block) = block_map.get(target.as_str()) {
                            self.emit_block_with_loops(next_block, block_map, loop_headers, emitted, module, output, depth);
                        }
                    }
                }
                Instruction::CondBranch { cond, then_label, else_label } => {
                    // Check if this is the condition of a for/while loop (we're inside the cond block)
                    if block.label.starts_with("for_cond_") || block.label.starts_with("while_cond_") {
                        // Skip - handled by loop emitter
                        continue;
                    }

                    // Regular if/else
                    writeln!(output, "{}if {} {{", indent, self.emit_value(cond)).unwrap();

                    if let Some(then_block) = block_map.get(then_label.as_str()) {
                        if !emitted.contains(then_label.as_str()) {
                            self.emit_block_with_loops(then_block, block_map, loop_headers, emitted, module, output, depth + 1);
                        }
                    }

                    if then_label != else_label {
                        writeln!(output, "{}}} else {{", indent).unwrap();
                        if let Some(else_block) = block_map.get(else_label.as_str()) {
                            if !emitted.contains(else_label.as_str()) {
                                self.emit_block_with_loops(else_block, block_map, loop_headers, emitted, module, output, depth + 1);
                            }
                        }
                    }

                    writeln!(output, "{}}}", indent).unwrap();
                }
                _ => {
                    self.emit_instruction_indented(inst, module, output, depth);
                }
            }
        }
    }

    fn emit_for_loop(
        &self,
        loop_info: &ForLoopInfo,
        block_map: &std::collections::HashMap<&str, &mendes_ir::BasicBlock>,
        emitted: &mut std::collections::HashSet<&str>,
        module: &Module,
        output: &mut String,
        depth: usize,
    ) {
        let indent = "    ".repeat(depth);
        let cond_label = format!("for_cond_{}", loop_info.loop_id);

        // Mark loop blocks as emitted
        emitted.insert(Box::leak(cond_label.clone().into_boxed_str()));
        emitted.insert(Box::leak(loop_info.body_label.clone().into_boxed_str()));
        emitted.insert(Box::leak(loop_info.inc_label.clone().into_boxed_str()));

        // Determine the comparison operator for Rust
        let cmp_str = match loop_info.cmp_op {
            CompareOp::Lt => "<",
            CompareOp::Le => "<=",
            CompareOp::Gt => ">",
            CompareOp::Ge => ">=",
            _ => "<",
        };

        // Generate Rust for loop
        // For now, generate a while loop that's equivalent
        writeln!(output, "{}while {} {} {} {{",
            indent,
            loop_info.loop_var,
            cmp_str,
            self.emit_value(&loop_info.end_val)
        ).unwrap();

        // Emit body block contents
        if let Some(body_block) = block_map.get(loop_info.body_label.as_str()) {
            for inst in &body_block.instructions {
                match inst {
                    Instruction::Branch { .. } => {
                        // Skip branch to inc block
                    }
                    _ => {
                        self.emit_instruction_indented(inst, module, output, depth + 1);
                    }
                }
            }
        }

        // Emit increment
        if let Some(inc_block) = block_map.get(loop_info.inc_label.as_str()) {
            for inst in &inc_block.instructions {
                match inst {
                    Instruction::Branch { .. } => {
                        // Skip branch back to cond
                    }
                    Instruction::Load { .. } => {
                        // Skip load of loop var
                    }
                    Instruction::Binary { dest, op, left: _, right } => {
                        // This is the increment operation
                        if *op == BinaryOp::Add && *right == Value::ConstInt(1) {
                            writeln!(output, "{}{} += 1;", "    ".repeat(depth + 1), loop_info.loop_var).unwrap();
                        } else {
                            writeln!(output, "{}let __t{} = {} {} {};",
                                "    ".repeat(depth + 1), dest, loop_info.loop_var,
                                self.emit_binary_op(op), self.emit_value(right)).unwrap();
                            writeln!(output, "{}{} = __t{};", "    ".repeat(depth + 1), loop_info.loop_var, dest).unwrap();
                        }
                    }
                    Instruction::Store { .. } => {
                        // Skip store - we handle it with +=
                    }
                    _ => {
                        self.emit_instruction_indented(inst, module, output, depth + 1);
                    }
                }
            }
        }

        writeln!(output, "{}}}", indent).unwrap();
    }

    fn emit_while_loop(
        &self,
        loop_info: &WhileLoopInfo,
        block_map: &std::collections::HashMap<&str, &mendes_ir::BasicBlock>,
        emitted: &mut std::collections::HashSet<&str>,
        module: &Module,
        output: &mut String,
        depth: usize,
    ) {
        let indent = "    ".repeat(depth);
        let cond_label = format!("while_cond_{}", loop_info.loop_id);

        // Mark loop blocks as emitted
        emitted.insert(Box::leak(cond_label.clone().into_boxed_str()));
        emitted.insert(Box::leak(loop_info.body_label.clone().into_boxed_str()));

        // Get condition from cond block
        let mut condition = String::from("true");
        if let Some(cond_block) = block_map.get(cond_label.as_str()) {
            for inst in &cond_block.instructions {
                if let Instruction::Compare { dest, op, left, right } = inst {
                    condition = format!("{} {} {}",
                        self.emit_value(left),
                        self.emit_compare_op(op),
                        self.emit_value(right));
                    break;
                }
            }
        }

        writeln!(output, "{}while {} {{", indent, condition).unwrap();

        // Emit body
        if let Some(body_block) = block_map.get(loop_info.body_label.as_str()) {
            for inst in &body_block.instructions {
                match inst {
                    Instruction::Branch { .. } => {}
                    _ => {
                        self.emit_instruction_indented(inst, module, output, depth + 1);
                    }
                }
            }
        }

        writeln!(output, "{}}}", indent).unwrap();
    }

    fn emit_instruction_indented(&self, inst: &Instruction, module: &Module, output: &mut String, depth: usize) {
        let indent = "    ".repeat(depth);
        match inst {
            Instruction::Alloca { .. } => {
                // Skip - already declared
            }

            Instruction::Store { value, ptr } => {
                writeln!(output, "{}{} = {};", indent, self.emit_value(ptr), self.emit_value(value)).unwrap();
            }

            Instruction::Load { dest, ptr, .. } => {
                writeln!(output, "{}let __t{} = {}.clone();", indent, dest, self.emit_value(ptr)).unwrap();
            }

            Instruction::Binary { dest, op, left, right } => {
                writeln!(output, "{}let __t{} = {} {} {};",
                    indent, dest, self.emit_value(left), self.emit_binary_op(op), self.emit_value(right)).unwrap();
            }

            Instruction::Compare { dest, op, left, right } => {
                writeln!(output, "{}let __t{} = {} {} {};",
                    indent, dest, self.emit_value(left), self.emit_compare_op(op), self.emit_value(right)).unwrap();
            }

            Instruction::Not { dest, value } => {
                writeln!(output, "{}let __t{} = !{};", indent, dest, self.emit_value(value)).unwrap();
            }

            Instruction::Neg { dest, value } => {
                writeln!(output, "{}let __t{} = -{};", indent, dest, self.emit_value(value)).unwrap();
            }

            Instruction::Call { dest, func, args } => {
                let args_str: Vec<String> = args.iter().map(|a| self.emit_value(a)).collect();

                // Check for built-in functions
                match func.as_str() {
                    "print" | "__builtin_print" => {
                        writeln!(output, "{}print!(\"{{}}\", {});", indent, args_str.join(", ")).unwrap();
                        return;
                    }
                    "println" | "__builtin_println" => {
                        if args_str.is_empty() {
                            writeln!(output, "{}println!();", indent).unwrap();
                        } else {
                            writeln!(output, "{}println!(\"{{}}\", {});", indent, args_str.join(", ")).unwrap();
                        }
                        return;
                    }
                    "log" | "__builtin_log" => {
                        writeln!(output, "{}eprintln!(\"[LOG] {{}}\", {});", indent, args_str.join(", ")).unwrap();
                        return;
                    }
                    "len" | "__builtin_len" => {
                        if let Some(d) = dest {
                            writeln!(output, "{}let __t{} = {}.len() as i64;", indent, d, args_str[0]).unwrap();
                        }
                        return;
                    }
                    "str" | "__builtin_str" => {
                        if let Some(d) = dest {
                            writeln!(output, "{}let __t{} = MendesString::new(&format!(\"{{}}\", {}));", indent, d, args_str[0]).unwrap();
                        }
                        return;
                    }
                    "int" | "__builtin_int" => {
                        if let Some(d) = dest {
                            writeln!(output, "{}let __t{} = {}.as_str().parse::<i64>().unwrap_or(0);", indent, d, args_str[0]).unwrap();
                        }
                        return;
                    }
                    "float" | "__builtin_float" => {
                        if let Some(d) = dest {
                            writeln!(output, "{}let __t{} = {}.as_str().parse::<f64>().unwrap_or(0.0);", indent, d, args_str[0]).unwrap();
                        }
                        return;
                    }
                    // Result/Option constructors
                    "__result_ok" => {
                        if let Some(d) = dest {
                            writeln!(output, "{}let __t{} = MendesResult::Ok({});", indent, d, args_str[0]).unwrap();
                        }
                        return;
                    }
                    "__result_err" => {
                        if let Some(d) = dest {
                            writeln!(output, "{}let __t{} = MendesResult::Err({});", indent, d, args_str[0]).unwrap();
                        }
                        return;
                    }
                    "__option_some" => {
                        if let Some(d) = dest {
                            writeln!(output, "{}let __t{} = MendesOption::Some({});", indent, d, args_str[0]).unwrap();
                        }
                        return;
                    }
                    // Pattern matching helpers
                    "__is_none" => {
                        if let Some(d) = dest {
                            writeln!(output, "{}let __t{} = {}.is_none();", indent, d, args_str[0]).unwrap();
                        }
                        return;
                    }
                    "__is_some" => {
                        if let Some(d) = dest {
                            writeln!(output, "{}let __t{} = {}.is_some();", indent, d, args_str[0]).unwrap();
                        }
                        return;
                    }
                    "__is_ok" => {
                        if let Some(d) = dest {
                            writeln!(output, "{}let __t{} = {}.is_ok();", indent, d, args_str[0]).unwrap();
                        }
                        return;
                    }
                    "__is_err" => {
                        if let Some(d) = dest {
                            writeln!(output, "{}let __t{} = {}.is_err();", indent, d, args_str[0]).unwrap();
                        }
                        return;
                    }
                    "__extract_variant_field" => {
                        // Extract field from variant (simplified - assumes unwrap is safe after pattern match)
                        if let Some(d) = dest {
                            writeln!(output, "{}let __t{} = {}.unwrap_field({});", indent, d, args_str[0], args_str[1]).unwrap();
                        }
                        return;
                    }
                    "__unwrap" => {
                        if let Some(d) = dest {
                            writeln!(output, "{}let __t{} = {}.unwrap();", indent, d, args_str[0]).unwrap();
                        }
                        return;
                    }
                    // Try operator (?) helpers
                    "__try_is_ok" => {
                        if let Some(d) = dest {
                            writeln!(output, "{}let __t{} = {}.is_ok();", indent, d, args_str[0]).unwrap();
                        }
                        return;
                    }
                    "__try_propagate" => {
                        if let Some(d) = dest {
                            // Convert error type for propagation
                            writeln!(output, "{}let __t{} = {}.map_err(|e| e.into()).err().unwrap();", indent, d, args_str[0]).unwrap();
                        }
                        return;
                    }
                    "__try_unwrap" => {
                        if let Some(d) = dest {
                            writeln!(output, "{}let __t{} = {}.unwrap();", indent, d, args_str[0]).unwrap();
                        }
                        return;
                    }
                    // String interpolation
                    "__string_format" => {
                        if let Some(d) = dest {
                            // Generate format! call
                            if args_str.is_empty() {
                                writeln!(output, "{}let __t{} = MendesString::new(\"\");", indent, d).unwrap();
                            } else {
                                // Build format string with placeholders
                                let format_args: Vec<_> = args_str.iter().map(|s| format!("{{{}}}", s)).collect();
                                writeln!(output, "{}let __t{} = MendesString::new(&format!(\"{}\"));",
                                    indent, d, format_args.join("")).unwrap();
                            }
                        }
                        return;
                    }
                    _ => {}
                }

                if let Some(d) = dest {
                    writeln!(output, "{}let __t{} = {}({});", indent, d, func, args_str.join(", ")).unwrap();
                } else {
                    writeln!(output, "{}{}({});", indent, func, args_str.join(", ")).unwrap();
                }
            }

            Instruction::Return(value) => {
                match value {
                    Value::Void => writeln!(output, "{}return;", indent).unwrap(),
                    _ => writeln!(output, "{}return {};", indent, self.emit_value(value)).unwrap(),
                }
            }

            Instruction::Await { dest, future } => {
                writeln!(output, "{}let __t{} = {}.await;", indent, dest, self.emit_value(future)).unwrap();
            }

            Instruction::GetField { dest, ptr, struct_name, field_index, field_name } => {
                // Try to look up from module first, fall back to field_name from instruction
                let resolved_name = module.get_struct(struct_name)
                    .and_then(|s| s.fields.get(*field_index))
                    .map(|(name, _)| name.as_str())
                    .unwrap_or(field_name.as_str());
                writeln!(output, "{}let __t{} = {}.{};", indent, dest, self.emit_value(ptr), resolved_name).unwrap();
            }

            Instruction::SetField { ptr, struct_name, field_index, field_name, value } => {
                // Try to look up from module first, fall back to field_name from instruction
                let resolved_name = module.get_struct(struct_name)
                    .and_then(|s| s.fields.get(*field_index))
                    .map(|(name, _)| name.as_str())
                    .unwrap_or(field_name.as_str());
                writeln!(output, "{}{}.{} = {};", indent, self.emit_value(ptr), resolved_name, self.emit_value(value)).unwrap();
            }

            Instruction::GetElement { dest, ptr, index } => {
                writeln!(output, "{}let __t{} = {}[{} as usize];", indent, dest, self.emit_value(ptr), self.emit_value(index)).unwrap();
            }

            Instruction::SetElement { ptr, index, value } => {
                writeln!(output, "{}{}[{} as usize] = {};", indent, self.emit_value(ptr), self.emit_value(index), self.emit_value(value)).unwrap();
            }

            Instruction::NewStruct { dest, struct_name } => {
                // Generate proper struct initialization with Default or zeroed values
                if let Some(struct_def) = module.get_struct(struct_name) {
                    write!(output, "{}let mut __t{} = {} {{ ", indent, dest, struct_name).unwrap();
                    for (i, (field_name, field_type)) in struct_def.fields.iter().enumerate() {
                        if i > 0 {
                            write!(output, ", ").unwrap();
                        }
                        let default_val = match field_type {
                            IrType::I64 => "0",
                            IrType::F64 => "0.0",
                            IrType::Bool => "false",
                            IrType::String => "MendesString::new(\"\")",
                            _ => "Default::default()",
                        };
                        write!(output, "{}: {}", field_name, default_val).unwrap();
                    }
                    writeln!(output, " }};").unwrap();
                } else {
                    writeln!(output, "{}let mut __t{} = {} {{ /* unknown struct */ }};", indent, dest, struct_name).unwrap();
                }
            }

            Instruction::NewArray { dest, elem_type, size } => {
                writeln!(output, "{}let __t{}: Vec<{}> = Vec::with_capacity({} as usize);",
                    indent, dest, self.emit_type(elem_type), self.emit_value(size)).unwrap();
            }

            Instruction::Cast { dest, value, to_type } => {
                writeln!(output, "{}let __t{} = {} as {};", indent, dest, self.emit_value(value), self.emit_type(to_type)).unwrap();
            }

            Instruction::Comment(text) => {
                writeln!(output, "{}// {}", indent, text).unwrap();
            }

            // Skip control flow - handled separately
            Instruction::Branch { .. } | Instruction::CondBranch { .. } | Instruction::Phi { .. } => {}
        }
    }

    fn emit_route_setup(&self, module: &Module, output: &mut String) {
        if module.routes.is_empty() {
            return;
        }

        // Check if we have databases to pass to handlers
        let has_db = !module.databases.is_empty();

        if has_db {
            writeln!(output, "fn setup_routes(router: &mut Router, db: Arc<DbContext>) {{").unwrap();
        } else {
            writeln!(output, "fn setup_routes(router: &mut Router) {{").unwrap();
        }

        for route in &module.routes {
            let method = route.method.to_lowercase();
            let handler = &route.handler;

            // Extract path params info
            let path_params = self.extract_path_params(&route.path);

            // Clone db for closure if needed
            if has_db {
                writeln!(output, "    let db = db.clone();").unwrap();
            }

            // Wrap handler in closure that takes Request
            writeln!(output, "    router.{}(\"{}\", move |req| {{", method, route.path).unwrap();
            if has_db {
                writeln!(output, "        let db = db.clone();").unwrap();
            }
            writeln!(output, "        async move {{").unwrap();

            // Extract path parameters
            for (param_name, param_type) in &path_params {
                match param_type.as_str() {
                    "int" => {
                        writeln!(output, "            let {}: i64 = req.param_int(\"{}\").unwrap_or(0);",
                            param_name, param_name).unwrap();
                    }
                    "string" => {
                        writeln!(output, "            let {}: MendesString = MendesString::new(&req.param(\"{}\").cloned().unwrap_or_default());",
                            param_name, param_name).unwrap();
                    }
                    _ => {
                        writeln!(output, "            let {}: String = req.param(\"{}\").cloned().unwrap_or_default();",
                            param_name, param_name).unwrap();
                    }
                }
            }

            // Apply middlewares (if any)
            for middleware_name in &route.middlewares {
                writeln!(output, "            // Middleware: {}", middleware_name).unwrap();
                writeln!(output, "            if let Some(resp) = __middleware_{}(&req).await {{", middleware_name).unwrap();
                writeln!(output, "                return resp;").unwrap();
                writeln!(output, "            }}").unwrap();
            }

            // Call handler
            if route.is_async {
                if has_db && path_params.is_empty() {
                    writeln!(output, "            let result = {}(req, &db).await;", handler).unwrap();
                } else if has_db {
                    let params: Vec<_> = path_params.iter().map(|(n, _)| n.clone()).collect();
                    writeln!(output, "            let result = {}(req, {}, &db).await;",
                        handler, params.join(", ")).unwrap();
                } else if path_params.is_empty() {
                    writeln!(output, "            let result = {}(req).await;", handler).unwrap();
                } else {
                    let params: Vec<_> = path_params.iter().map(|(n, _)| n.clone()).collect();
                    writeln!(output, "            let result = {}(req, {}).await;",
                        handler, params.join(", ")).unwrap();
                }
            } else {
                if path_params.is_empty() {
                    writeln!(output, "            let result = {}(req);", handler).unwrap();
                } else {
                    let params: Vec<_> = path_params.iter().map(|(n, _)| n.clone()).collect();
                    writeln!(output, "            let result = {}(req, {});",
                        handler, params.join(", ")).unwrap();
                }
            }

            writeln!(output, "            Response::from(result)").unwrap();
            writeln!(output, "        }}").unwrap();
            writeln!(output, "    }});").unwrap();
        }

        // WebSocket routes
        for ws_route in &module.websocket_routes {
            let path_params = self.extract_path_params(&ws_route.path);

            writeln!(output).unwrap();
            writeln!(output, "    // WebSocket endpoint: {}", ws_route.path).unwrap();
            writeln!(output, "    router.ws(\"{}\", move |ws| {{", ws_route.path).unwrap();
            writeln!(output, "        async move {{").unwrap();

            // on_connect handler
            if let Some(connect_handler) = &ws_route.on_connect {
                writeln!(output, "            // on_connect").unwrap();
                if path_params.is_empty() {
                    writeln!(output, "            {}(ws.clone()).await;", connect_handler).unwrap();
                } else {
                    let params: Vec<_> = path_params.iter().map(|(n, _)| n.clone()).collect();
                    writeln!(output, "            {}(ws.clone(), {}).await;", connect_handler, params.join(", ")).unwrap();
                }
            }

            // on_message handler
            if let Some(message_handler) = &ws_route.on_message {
                writeln!(output, "            // Message loop").unwrap();
                writeln!(output, "            while let Some(msg) = ws.recv().await {{").unwrap();
                if path_params.is_empty() {
                    writeln!(output, "                {}(ws.clone(), msg).await;", message_handler).unwrap();
                } else {
                    let params: Vec<_> = path_params.iter().map(|(n, _)| n.clone()).collect();
                    writeln!(output, "                {}(ws.clone(), msg, {}).await;", message_handler, params.join(", ")).unwrap();
                }
                writeln!(output, "            }}").unwrap();
            }

            // on_disconnect handler
            if let Some(disconnect_handler) = &ws_route.on_disconnect {
                writeln!(output, "            // on_disconnect").unwrap();
                if path_params.is_empty() {
                    writeln!(output, "            {}(ws.clone()).await;", disconnect_handler).unwrap();
                } else {
                    let params: Vec<_> = path_params.iter().map(|(n, _)| n.clone()).collect();
                    writeln!(output, "            {}(ws.clone(), {}).await;", disconnect_handler, params.join(", ")).unwrap();
                }
            }

            writeln!(output, "        }}").unwrap();
            writeln!(output, "    }});").unwrap();
        }

        writeln!(output, "}}").unwrap();
        writeln!(output).unwrap();
    }

    /// Extract path parameters from route path like /users/{id:int}
    fn extract_path_params(&self, path: &str) -> Vec<(String, String)> {
        let mut params = Vec::new();
        let mut chars = path.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '{' {
                let mut name = String::new();
                let mut ty = String::new();
                let mut in_type = false;

                while let Some(&c) = chars.peek() {
                    chars.next();
                    if c == '}' {
                        break;
                    } else if c == ':' {
                        in_type = true;
                    } else if in_type {
                        ty.push(c);
                    } else {
                        name.push(c);
                    }
                }

                if !name.is_empty() {
                    params.push((name, if ty.is_empty() { "string".to_string() } else { ty }));
                }
            }
        }

        params
    }

    fn emit_db_context(&self, module: &Module, output: &mut String) {
        if module.databases.is_empty() {
            return;
        }

        writeln!(output, "/// Database context holding all connection pools").unwrap();
        writeln!(output, "#[derive(Clone)]").unwrap();
        writeln!(output, "pub struct DbContext {{").unwrap();
        for db in &module.databases {
            let pool_type = match db.db_type.as_str() {
                "postgres" => "PostgresPool",
                "mysql" => "MysqlPool",
                "sqlite" => "SqlitePool",
                _ => "PostgresPool",
            };
            writeln!(output, "    pub {}: Option<{}>,", db.name, pool_type).unwrap();
        }
        writeln!(output, "}}").unwrap();
        writeln!(output).unwrap();

        writeln!(output, "impl DbContext {{").unwrap();
        writeln!(output, "    pub async fn new() -> Self {{").unwrap();
        writeln!(output, "        Self {{").unwrap();
        for db in &module.databases {
            writeln!(output, "            {}: None, // TODO: Initialize from env", db.name).unwrap();
        }
        writeln!(output, "        }}").unwrap();
        writeln!(output, "    }}").unwrap();
        writeln!(output, "}}").unwrap();
        writeln!(output).unwrap();
    }

    fn emit_main(&self, module: &Module, output: &mut String) {
        let has_db = !module.databases.is_empty();

        writeln!(output, "#[tokio::main]").unwrap();
        writeln!(output, "async fn main() {{").unwrap();
        writeln!(output, "    mendes_runtime::init();").unwrap();
        writeln!(output).unwrap();

        // Database connections
        if has_db {
            writeln!(output, "    // Initialize database connections").unwrap();
            writeln!(output, "    let db = Arc::new(DbContext::new().await);").unwrap();
            for db in &module.databases {
                writeln!(output, "    // Database '{}': {} (pool_size: {})",
                    db.name, db.db_type, db.pool_size).unwrap();
            }
            writeln!(output).unwrap();
        }

        // Router setup
        writeln!(output, "    let mut router = Router::new();").unwrap();
        if !module.routes.is_empty() {
            if has_db {
                writeln!(output, "    setup_routes(&mut router, db);").unwrap();
            } else {
                writeln!(output, "    setup_routes(&mut router);").unwrap();
            }
        }
        writeln!(output).unwrap();

        // Server
        if let Some(server) = &module.server {
            writeln!(output, "    println!(\"Starting Mendes server on {}:{}\");", server.host, server.port).unwrap();
            writeln!(output, "    println!(\"Routes:\");").unwrap();
            for route in &module.routes {
                // Escape curly braces for println!
                let escaped_path = route.path.replace('{', "{{").replace('}', "}}");
                writeln!(output, "    println!(\"  {} {}\");", route.method, escaped_path).unwrap();
            }
            // WebSocket routes
            for ws_route in &module.websocket_routes {
                let escaped_path = ws_route.path.replace('{', "{{").replace('}', "}}");
                writeln!(output, "    println!(\"  WS {}\");", escaped_path).unwrap();
            }
            writeln!(output).unwrap();
            writeln!(output, "    Server::new(\"{}:{}\")", server.host, server.port).unwrap();
            writeln!(output, "        .router(router)").unwrap();
            writeln!(output, "        .run()").unwrap();
            writeln!(output, "        .await").unwrap();
            writeln!(output, "        .expect(\"Server error\");").unwrap();
        } else {
            writeln!(output, "    println!(\"No server configuration found\");").unwrap();
        }

        writeln!(output, "}}").unwrap();
    }
}

impl CodeGen for RustBackend {
    type Output = String;

    fn generate(&self, module: &Module) -> String {
        let mut output = String::new();

        self.emit_prelude(module, &mut output);
        self.emit_string_table(module, &mut output);
        self.emit_type_aliases(module, &mut output);
        self.emit_structs(module, &mut output);
        self.emit_traits(module, &mut output);
        self.emit_db_context(module, &mut output);

        // Generate functions (skip impl methods - they are generated inline)
        let impl_methods: std::collections::HashSet<&str> = module.impls.iter()
            .flat_map(|i| i.methods.iter().map(|s| s.as_str()))
            .collect();

        for func in &module.functions {
            if !impl_methods.contains(func.name.as_str()) {
                self.emit_function(func, module, &mut output);
            }
        }

        // Generate trait implementations
        self.emit_impls(module, &mut output);

        self.emit_route_setup(module, &mut output);
        self.emit_main(module, &mut output);

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mendes_ir::{Module, Function, IrType, Instruction, Value};

    #[test]
    fn test_simple_function() {
        let mut module = Module::new("test");

        let mut func = Function::new("hello", IrType::String, false);
        func.emit(Instruction::Return(Value::ConstString(0)));
        module.add_function(func);
        module.add_string("Hello, World!".to_string());

        let backend = RustBackend::new();
        let code = backend.generate(&module);

        assert!(code.contains("fn hello()"));
        assert!(code.contains("fn __str_0()"));
        assert!(code.contains("Hello, World!"));
    }

    #[test]
    fn test_struct_generation() {
        let mut module = Module::new("test");

        let mut user = mendes_ir::types::StructDef::new("User".to_string());
        user.add_field("id".to_string(), IrType::I64);
        user.add_field("name".to_string(), IrType::String);
        module.add_struct(user);

        let backend = RustBackend::new();
        let code = backend.generate(&module);

        assert!(code.contains("pub struct User"));
        assert!(code.contains("pub id: i64"));
        assert!(code.contains("pub name: MendesString"));
    }
}
