//! IR Instructions
//!
//! Low-level instructions in simplified SSA format.

use crate::types::IrType;
use std::fmt;

/// Value identifier (SSA)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Value {
    /// Integer constant
    ConstInt(i64),
    /// Float constant
    ConstFloat(u64), // f64 bits
    /// Boolean constant
    ConstBool(bool),
    /// String constant (index in string table)
    ConstString(usize),
    /// Local variable (by temporary name %0, %1, etc)
    Local(String),
    /// Function parameter
    Param(usize),
    /// Global (by name @name)
    Global(String),
    /// Result of previous instruction
    Temp(u32),
    /// Void / no value
    Void,
}

impl Value {
    pub fn const_int(v: i64) -> Self {
        Value::ConstInt(v)
    }

    pub fn const_float(v: f64) -> Self {
        Value::ConstFloat(v.to_bits())
    }

    pub fn const_bool(v: bool) -> Self {
        Value::ConstBool(v)
    }

    pub fn local(name: impl Into<String>) -> Self {
        Value::Local(name.into())
    }

    pub fn temp(id: u32) -> Self {
        Value::Temp(id)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::ConstInt(v) => write!(f, "{}", v),
            Value::ConstFloat(bits) => write!(f, "{}", f64::from_bits(*bits)),
            Value::ConstBool(v) => write!(f, "{}", v),
            Value::ConstString(idx) => write!(f, "str#{}", idx),
            Value::Local(name) => write!(f, "%{}", name),
            Value::Param(idx) => write!(f, "%arg{}", idx),
            Value::Global(name) => write!(f, "@{}", name),
            Value::Temp(id) => write!(f, "%t{}", id),
            Value::Void => write!(f, "void"),
        }
    }
}

/// Binary operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    // Bitwise
    And,
    Or,
    Xor,
    Shl,
    Shr,
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinaryOp::Add => write!(f, "add"),
            BinaryOp::Sub => write!(f, "sub"),
            BinaryOp::Mul => write!(f, "mul"),
            BinaryOp::Div => write!(f, "div"),
            BinaryOp::Mod => write!(f, "mod"),
            BinaryOp::And => write!(f, "and"),
            BinaryOp::Or => write!(f, "or"),
            BinaryOp::Xor => write!(f, "xor"),
            BinaryOp::Shl => write!(f, "shl"),
            BinaryOp::Shr => write!(f, "shr"),
        }
    }
}

/// Comparison operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

impl fmt::Display for CompareOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompareOp::Eq => write!(f, "eq"),
            CompareOp::Ne => write!(f, "ne"),
            CompareOp::Lt => write!(f, "lt"),
            CompareOp::Le => write!(f, "le"),
            CompareOp::Gt => write!(f, "gt"),
            CompareOp::Ge => write!(f, "ge"),
        }
    }
}

/// IR Instruction
#[derive(Debug, Clone)]
pub enum Instruction {
    /// Allocates space on the stack for a local variable
    /// %name = alloca type
    Alloca {
        dest: String,
        ty: IrType,
    },

    /// Stores value in pointer
    /// store value, ptr
    Store {
        value: Value,
        ptr: Value,
    },

    /// Loads value from pointer
    /// %dest = load ptr
    Load {
        dest: u32,
        ptr: Value,
        ty: IrType,
    },

    /// Binary operation
    /// %dest = op left, right
    Binary {
        dest: u32,
        op: BinaryOp,
        left: Value,
        right: Value,
    },

    /// Comparison
    /// %dest = cmp op left, right
    Compare {
        dest: u32,
        op: CompareOp,
        left: Value,
        right: Value,
    },

    /// Logical negation
    /// %dest = not value
    Not {
        dest: u32,
        value: Value,
    },

    /// Arithmetic negation
    /// %dest = neg value
    Neg {
        dest: u32,
        value: Value,
    },

    /// Function call
    /// %dest = call @func(args...)
    Call {
        dest: Option<u32>,
        func: String,
        args: Vec<Value>,
    },

    /// Return
    /// ret value
    Return(Value),

    /// Unconditional branch
    /// br label
    Branch {
        target: String,
    },

    /// Conditional branch
    /// br cond, then_label, else_label
    CondBranch {
        cond: Value,
        then_label: String,
        else_label: String,
    },

    /// Phi node (SSA)
    /// %dest = phi [val1, label1], [val2, label2], ...
    Phi {
        dest: u32,
        incoming: Vec<(Value, String)>,
    },

    /// Struct field access
    /// %dest = getfield ptr, field_index
    GetField {
        dest: u32,
        ptr: Value,
        struct_name: String,
        field_index: usize,
        /// Field name (for codegen when struct lookup might fail)
        field_name: String,
    },

    /// Struct field write
    /// setfield ptr, field_index, value
    SetField {
        ptr: Value,
        struct_name: String,
        field_index: usize,
        /// Field name (for codegen when struct lookup might fail)
        field_name: String,
        value: Value,
    },

    /// Array element access
    /// %dest = getelem ptr, index
    GetElement {
        dest: u32,
        ptr: Value,
        index: Value,
    },

    /// Array element write
    /// setelem ptr, index, value
    SetElement {
        ptr: Value,
        index: Value,
        value: Value,
    },

    /// Await of future (will be expanded into state machine)
    /// %dest = await future
    Await {
        dest: u32,
        future: Value,
    },

    /// Creates a new struct
    /// %dest = newstruct StructName
    NewStruct {
        dest: u32,
        struct_name: String,
    },

    /// Creates a new array
    /// %dest = newarray elem_type, size
    NewArray {
        dest: u32,
        elem_type: IrType,
        size: Value,
    },

    /// Type conversion
    /// %dest = cast value to type
    Cast {
        dest: u32,
        value: Value,
        to_type: IrType,
    },

    /// Comment / debug info
    Comment(String),
}

impl Instruction {
    /// Returns the instruction's destination (if any)
    pub fn dest(&self) -> Option<u32> {
        match self {
            Instruction::Load { dest, .. } => Some(*dest),
            Instruction::Binary { dest, .. } => Some(*dest),
            Instruction::Compare { dest, .. } => Some(*dest),
            Instruction::Not { dest, .. } => Some(*dest),
            Instruction::Neg { dest, .. } => Some(*dest),
            Instruction::Call { dest, .. } => *dest,
            Instruction::Phi { dest, .. } => Some(*dest),
            Instruction::GetField { dest, .. } => Some(*dest),
            Instruction::GetElement { dest, .. } => Some(*dest),
            Instruction::Await { dest, .. } => Some(*dest),
            Instruction::NewStruct { dest, .. } => Some(*dest),
            Instruction::NewArray { dest, .. } => Some(*dest),
            Instruction::Cast { dest, .. } => Some(*dest),
            _ => None,
        }
    }

    /// Checks if it is a block terminator instruction
    pub fn is_terminator(&self) -> bool {
        matches!(self, Instruction::Return(_) | Instruction::Branch { .. } | Instruction::CondBranch { .. })
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::Alloca { dest, ty } => {
                write!(f, "%{} = alloca {}", dest, ty)
            }
            Instruction::Store { value, ptr } => {
                write!(f, "store {}, {}", value, ptr)
            }
            Instruction::Load { dest, ptr, ty } => {
                write!(f, "%t{} = load {} {}", dest, ty, ptr)
            }
            Instruction::Binary { dest, op, left, right } => {
                write!(f, "%t{} = {} {}, {}", dest, op, left, right)
            }
            Instruction::Compare { dest, op, left, right } => {
                write!(f, "%t{} = cmp {} {}, {}", dest, op, left, right)
            }
            Instruction::Not { dest, value } => {
                write!(f, "%t{} = not {}", dest, value)
            }
            Instruction::Neg { dest, value } => {
                write!(f, "%t{} = neg {}", dest, value)
            }
            Instruction::Call { dest, func, args } => {
                if let Some(d) = dest {
                    write!(f, "%t{} = ", d)?;
                }
                write!(f, "call @{}(", func)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            }
            Instruction::Return(value) => {
                write!(f, "ret {}", value)
            }
            Instruction::Branch { target } => {
                write!(f, "br {}", target)
            }
            Instruction::CondBranch { cond, then_label, else_label } => {
                write!(f, "br {}, {}, {}", cond, then_label, else_label)
            }
            Instruction::Phi { dest, incoming } => {
                write!(f, "%t{} = phi ", dest)?;
                for (i, (val, label)) in incoming.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "[{}, {}]", val, label)?;
                }
                Ok(())
            }
            Instruction::GetField { dest, ptr, struct_name, field_index, field_name } => {
                write!(f, "%t{} = getfield {} %{}.{} ({})", dest, ptr, struct_name, field_index, field_name)
            }
            Instruction::SetField { ptr, struct_name, field_index, field_name, value } => {
                write!(f, "setfield {} %{}.{} ({}), {}", ptr, struct_name, field_index, field_name, value)
            }
            Instruction::GetElement { dest, ptr, index } => {
                write!(f, "%t{} = getelem {}, {}", dest, ptr, index)
            }
            Instruction::SetElement { ptr, index, value } => {
                write!(f, "setelem {}, {}, {}", ptr, index, value)
            }
            Instruction::Await { dest, future } => {
                write!(f, "%t{} = await {}", dest, future)
            }
            Instruction::NewStruct { dest, struct_name } => {
                write!(f, "%t{} = newstruct %{}", dest, struct_name)
            }
            Instruction::NewArray { dest, elem_type, size } => {
                write!(f, "%t{} = newarray {}, {}", dest, elem_type, size)
            }
            Instruction::Cast { dest, value, to_type } => {
                write!(f, "%t{} = cast {} to {}", dest, value, to_type)
            }
            Instruction::Comment(text) => {
                write!(f, "; {}", text)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instruction_display() {
        let inst = Instruction::Binary {
            dest: 0,
            op: BinaryOp::Add,
            left: Value::Local("x".to_string()),
            right: Value::ConstInt(10),
        };
        assert_eq!(inst.to_string(), "%t0 = add %x, 10");
    }

    #[test]
    fn test_call_display() {
        let inst = Instruction::Call {
            dest: Some(1),
            func: "foo".to_string(),
            args: vec![Value::ConstInt(1), Value::ConstInt(2)],
        };
        assert_eq!(inst.to_string(), "%t1 = call @foo(1, 2)");
    }
}
