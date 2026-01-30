//! LLVM Backend for the Mendes language
//!
//! This module generates native code directly via LLVM.
//! Requires LLVM 17 to be installed on the system.
//!
//! # Setup
//!
//! 1. Install LLVM 17 from https://llvm.org/releases/
//! 2. Set environment variable: LLVM_SYS_170_PREFIX=C:\path\to\llvm
//! 3. Build with: cargo build --features llvm
//!
//! # Example
//!
//! ```rust,ignore
//! use mendes_codegen::{LlvmBackend, CodeGen};
//! use mendes_ir::Module;
//!
//! let module: Module = /* ... */;
//! let backend = LlvmBackend::new();
//! backend.compile(&module, "output.exe");
//! ```

use inkwell::context::Context;
use inkwell::module::Module as LlvmModule;
use inkwell::builder::Builder;
use inkwell::values::{BasicValueEnum, FunctionValue, PointerValue};
use inkwell::types::BasicTypeEnum;
use inkwell::targets::{Target, TargetMachine, InitializationConfig, RelocMode, CodeModel};
use inkwell::OptimizationLevel;
use inkwell::AddressSpace;
use std::collections::HashMap;

use mendes_ir::{
    Module as IrModule,
    Function,
    Instruction,
    Value as IrValue,
    IrType,
    BinaryOp,
    CompareOp,
};

use crate::CodeGen;

/// LLVM backend for native code generation
pub struct LlvmBackend<'ctx> {
    context: &'ctx Context,
    module: LlvmModule<'ctx>,
    builder: Builder<'ctx>,
    /// Mapping from IR names to LLVM values
    values: HashMap<String, BasicValueEnum<'ctx>>,
    /// Mapping from IR functions to LLVM functions
    functions: HashMap<String, FunctionValue<'ctx>>,
    /// String constants
    strings: HashMap<usize, PointerValue<'ctx>>,
}

impl<'ctx> LlvmBackend<'ctx> {
    /// Creates a new LLVM backend
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();

        Self {
            context,
            module,
            builder,
            values: HashMap::new(),
            functions: HashMap::new(),
            strings: HashMap::new(),
        }
    }

    /// Compiles an IR module to native code
    pub fn compile(&mut self, ir_module: &IrModule, output_path: &str) -> Result<(), String> {
        // Declare all functions first (for forward references)
        self.declare_functions(ir_module);

        // Emit string constants
        self.emit_string_table(ir_module);

        // Emit struct definitions
        self.emit_structs(ir_module);

        // Emit function bodies
        for func in &ir_module.functions {
            self.emit_function(func);
        }

        // Verify the module
        if let Err(msg) = self.module.verify() {
            return Err(format!("LLVM verification failed: {}", msg.to_string()));
        }

        // Initialize target
        Target::initialize_native(&InitializationConfig::default())
            .map_err(|e| format!("Failed to initialize target: {}", e))?;

        let triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&triple)
            .map_err(|e| format!("Failed to get target: {}", e.to_string()))?;

        let machine = target
            .create_target_machine(
                &triple,
                "generic",
                "",
                OptimizationLevel::Default,
                RelocMode::Default,
                CodeModel::Default,
            )
            .ok_or("Failed to create target machine")?;

        // Write object file
        let path = std::path::Path::new(output_path);
        machine
            .write_to_file(&self.module, inkwell::targets::FileType::Object, path)
            .map_err(|e| format!("Failed to write object file: {}", e.to_string()))?;

        Ok(())
    }

    /// Emits LLVM IR as a string (for debugging)
    pub fn emit_ir(&self) -> String {
        self.module.print_to_string().to_string()
    }

    fn declare_functions(&mut self, ir_module: &IrModule) {
        for func in &ir_module.functions {
            let fn_type = self.create_function_type(func);
            let fn_value = self.module.add_function(&func.name, fn_type, None);
            self.functions.insert(func.name.clone(), fn_value);
        }
    }

    fn create_function_type(&self, func: &Function) -> inkwell::types::FunctionType<'ctx> {
        let param_types: Vec<inkwell::types::BasicMetadataTypeEnum> = func.params
            .iter()
            .map(|(_, ty)| self.convert_type(ty).into())
            .collect();

        match self.convert_type(&func.return_type) {
            BasicTypeEnum::IntType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::FloatType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::PointerType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::StructType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::ArrayType(t) => t.fn_type(&param_types, false),
            BasicTypeEnum::VectorType(t) => t.fn_type(&param_types, false),
        }
    }

    fn convert_type(&self, ir_type: &IrType) -> BasicTypeEnum<'ctx> {
        match ir_type {
            IrType::I64 => self.context.i64_type().into(),
            IrType::F64 => self.context.f64_type().into(),
            IrType::Bool => self.context.bool_type().into(),
            IrType::String => self.context.ptr_type(AddressSpace::default()).into(),
            IrType::Void => self.context.i64_type().into(), // Represented as i64(0)
            IrType::Ptr(_) => self.context.ptr_type(AddressSpace::default()).into(),
            IrType::Array(elem, size) => {
                let elem_type = self.convert_type(elem);
                match elem_type {
                    BasicTypeEnum::IntType(t) => t.array_type(*size as u32).into(),
                    BasicTypeEnum::FloatType(t) => t.array_type(*size as u32).into(),
                    BasicTypeEnum::PointerType(t) => t.array_type(*size as u32).into(),
                    BasicTypeEnum::StructType(t) => t.array_type(*size as u32).into(),
                    _ => self.context.i64_type().array_type(*size as u32).into(),
                }
            }
            IrType::Struct(name) => {
                if let Some(struct_type) = self.module.get_struct_type(name) {
                    struct_type.into()
                } else {
                    self.context.opaque_struct_type(name).into()
                }
            }
            IrType::Function { .. } => self.context.ptr_type(AddressSpace::default()).into(),
            _ => self.context.i64_type().into(),
        }
    }

    fn emit_string_table(&mut self, ir_module: &IrModule) {
        for (idx, string) in ir_module.string_table.iter().enumerate() {
            let global_name = format!("__str_{}", idx);
            let string_value = self.context.const_string(string.as_bytes(), true);
            let global = self.module.add_global(
                string_value.get_type(),
                Some(AddressSpace::default()),
                &global_name
            );
            global.set_initializer(&string_value);
            global.set_constant(true);
            self.strings.insert(idx, global.as_pointer_value());
        }
    }

    fn emit_structs(&mut self, ir_module: &IrModule) {
        for struct_def in ir_module.structs.values() {
            let field_types: Vec<BasicTypeEnum> = struct_def.fields
                .iter()
                .map(|(_, ty)| self.convert_type(ty))
                .collect();

            let struct_type = self.context.opaque_struct_type(&struct_def.name);
            struct_type.set_body(&field_types, false);
        }
    }

    fn emit_function(&mut self, func: &Function) {
        let fn_value = *self.functions.get(&func.name).unwrap();
        self.values.clear();

        // Create entry block
        let entry = self.context.append_basic_block(fn_value, "entry");
        self.builder.position_at_end(entry);

        // Add parameters to values map
        for (i, (name, _)) in func.params.iter().enumerate() {
            let param = fn_value.get_nth_param(i as u32).unwrap();
            self.values.insert(name.clone(), param);
        }

        // Create blocks for all labels first
        let mut blocks: HashMap<String, inkwell::basic_block::BasicBlock> = HashMap::new();
        blocks.insert("entry".to_string(), entry);

        for block in &func.blocks {
            if block.label != "entry" {
                let llvm_block = self.context.append_basic_block(fn_value, &block.label);
                blocks.insert(block.label.clone(), llvm_block);
            }
        }

        // Emit instructions for each block
        for block in &func.blocks {
            self.builder.position_at_end(*blocks.get(&block.label).unwrap());

            for inst in &block.instructions {
                self.emit_instruction(inst, &blocks);
            }
        }
    }

    fn emit_instruction(
        &mut self,
        inst: &Instruction,
        blocks: &HashMap<String, inkwell::basic_block::BasicBlock<'ctx>>
    ) {
        match inst {
            Instruction::Alloca { dest, ty } => {
                let llvm_type = self.convert_type(ty);
                match llvm_type {
                    BasicTypeEnum::IntType(t) => {
                        let ptr = self.builder.build_alloca(t, dest).unwrap();
                        self.values.insert(dest.clone(), ptr.into());
                    }
                    BasicTypeEnum::FloatType(t) => {
                        let ptr = self.builder.build_alloca(t, dest).unwrap();
                        self.values.insert(dest.clone(), ptr.into());
                    }
                    BasicTypeEnum::PointerType(t) => {
                        let ptr = self.builder.build_alloca(t, dest).unwrap();
                        self.values.insert(dest.clone(), ptr.into());
                    }
                    BasicTypeEnum::StructType(t) => {
                        let ptr = self.builder.build_alloca(t, dest).unwrap();
                        self.values.insert(dest.clone(), ptr.into());
                    }
                    _ => {}
                }
            }

            Instruction::Store { value, ptr } => {
                let val = self.get_value(value);
                let ptr_val = self.get_value(ptr).into_pointer_value();
                self.builder.build_store(ptr_val, val).unwrap();
            }

            Instruction::Load { dest, ptr, ty } => {
                let llvm_type = self.convert_type(ty);
                let ptr_val = self.get_value(ptr).into_pointer_value();
                let loaded = self.builder.build_load(llvm_type, ptr_val, &format!("t{}", dest)).unwrap();
                self.values.insert(format!("__t{}", dest), loaded);
            }

            Instruction::Binary { dest, op, left, right } => {
                let lhs = self.get_value(left).into_int_value();
                let rhs = self.get_value(right).into_int_value();
                let name = format!("t{}", dest);

                let result: BasicValueEnum = match op {
                    BinaryOp::Add => self.builder.build_int_add(lhs, rhs, &name).unwrap().into(),
                    BinaryOp::Sub => self.builder.build_int_sub(lhs, rhs, &name).unwrap().into(),
                    BinaryOp::Mul => self.builder.build_int_mul(lhs, rhs, &name).unwrap().into(),
                    BinaryOp::Div => self.builder.build_int_signed_div(lhs, rhs, &name).unwrap().into(),
                    BinaryOp::Mod => self.builder.build_int_signed_rem(lhs, rhs, &name).unwrap().into(),
                    BinaryOp::And => self.builder.build_and(lhs, rhs, &name).unwrap().into(),
                    BinaryOp::Or => self.builder.build_or(lhs, rhs, &name).unwrap().into(),
                    BinaryOp::Xor => self.builder.build_xor(lhs, rhs, &name).unwrap().into(),
                    BinaryOp::Shl => self.builder.build_left_shift(lhs, rhs, &name).unwrap().into(),
                    BinaryOp::Shr => self.builder.build_right_shift(lhs, rhs, true, &name).unwrap().into(),
                };
                self.values.insert(format!("__t{}", dest), result);
            }

            Instruction::Compare { dest, op, left, right } => {
                let lhs = self.get_value(left).into_int_value();
                let rhs = self.get_value(right).into_int_value();

                let predicate = match op {
                    CompareOp::Eq => inkwell::IntPredicate::EQ,
                    CompareOp::Ne => inkwell::IntPredicate::NE,
                    CompareOp::Lt => inkwell::IntPredicate::SLT,
                    CompareOp::Le => inkwell::IntPredicate::SLE,
                    CompareOp::Gt => inkwell::IntPredicate::SGT,
                    CompareOp::Ge => inkwell::IntPredicate::SGE,
                };

                let result = self.builder.build_int_compare(predicate, lhs, rhs, &format!("t{}", dest)).unwrap();
                self.values.insert(format!("__t{}", dest), result.into());
            }

            Instruction::Call { dest, func: func_name, args } => {
                if let Some(callee) = self.functions.get(func_name).copied() {
                    let arg_values: Vec<inkwell::values::BasicMetadataValueEnum> = args
                        .iter()
                        .map(|a| self.get_value(a).into())
                        .collect();

                    let call = self.builder.build_call(callee, &arg_values, "call").unwrap();

                    if let Some(d) = dest {
                        if let Some(ret_val) = call.try_as_basic_value().left() {
                            self.values.insert(format!("__t{}", d), ret_val);
                        }
                    }
                }
            }

            Instruction::Return(value) => {
                match value {
                    IrValue::Void => {
                        self.builder.build_return(None).unwrap();
                    }
                    _ => {
                        let ret_val = self.get_value(value);
                        self.builder.build_return(Some(&ret_val)).unwrap();
                    }
                }
            }

            Instruction::Branch { target } => {
                if let Some(block) = blocks.get(target) {
                    self.builder.build_unconditional_branch(*block).unwrap();
                }
            }

            Instruction::CondBranch { cond, then_label, else_label } => {
                let cond_val = self.get_value(cond).into_int_value();

                if let (Some(then_block), Some(else_block)) =
                    (blocks.get(then_label), blocks.get(else_label))
                {
                    self.builder.build_conditional_branch(cond_val, *then_block, *else_block).unwrap();
                }
            }

            _ => {
                // Other instructions would be implemented here
            }
        }
    }

    fn get_value(&self, value: &IrValue) -> BasicValueEnum<'ctx> {
        match value {
            IrValue::ConstInt(n) => self.context.i64_type().const_int(*n as u64, true).into(),
            IrValue::ConstFloat(bits) => self.context.f64_type().const_float(f64::from_bits(*bits)).into(),
            IrValue::ConstBool(b) => self.context.bool_type().const_int(*b as u64, false).into(),
            IrValue::ConstString(idx) => {
                self.strings.get(idx).copied()
                    .map(|p| p.into())
                    .unwrap_or_else(|| self.context.i64_type().const_int(0, false).into())
            }
            IrValue::Local(name) => {
                self.values.get(name).copied()
                    .unwrap_or_else(|| self.context.i64_type().const_int(0, false).into())
            }
            IrValue::Temp(id) => {
                self.values.get(&format!("__t{}", id)).copied()
                    .unwrap_or_else(|| self.context.i64_type().const_int(0, false).into())
            }
            IrValue::Global(name) => {
                if let Some(func) = self.functions.get(name) {
                    func.as_global_value().as_pointer_value().into()
                } else {
                    self.context.i64_type().const_int(0, false).into()
                }
            }
            IrValue::Param(idx) => {
                let block = self.builder.get_insert_block().unwrap();
                let fn_value = block.get_parent().unwrap();
                fn_value.get_nth_param(*idx as u32)
                    .unwrap_or_else(|| self.context.i64_type().const_int(0, false).into())
            }
            IrValue::Void => self.context.i64_type().const_int(0, false).into(),
        }
    }
}

/// Wrapper to implement CodeGen trait
pub struct LlvmCodeGen;

impl CodeGen for LlvmCodeGen {
    type Output = Result<Vec<u8>, String>;

    fn generate(&self, module: &IrModule) -> Self::Output {
        let context = Context::create();
        let mut backend = LlvmBackend::new(&context, "mendes_module");

        // Generate to a temporary file
        let temp_path = std::env::temp_dir().join("mendes_output.o");
        backend.compile(module, temp_path.to_str().unwrap())?;

        // Read the object file
        std::fs::read(&temp_path).map_err(|e| format!("Failed to read output: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llvm_context() {
        let context = Context::create();
        let backend = LlvmBackend::new(&context, "test");
        let ir = backend.emit_ir();
        assert!(ir.contains("test"));
    }
}
