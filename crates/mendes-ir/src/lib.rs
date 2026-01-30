//! mendes-ir - Intermediate Representation of the Mendes language
//!
//! The Mendes IR is a low-level representation that:
//! - Represents async as state machines
//! - Represents HTTP handlers as functions
//! - Uses simplified SSA (Static Single Assignment)
//! - Prepares for LLVM IR generation
//!
//! # Architecture
//!
//! ```text
//! AST (mendes-parser)
//!         ↓
//!    [Lowering]
//!         ↓
//!   IR Module
//!   ├── Functions
//!   │   ├── Basic Blocks
//!   │   │   └── Instructions
//!   │   └── Local Variables
//!   ├── HTTP Routes
//!   ├── Globals
//!   └── Structs
//!         ↓
//!    [Codegen]
//!         ↓
//!   LLVM IR (mendes-codegen)
//! ```

pub mod types;
pub mod instruction;
pub mod module;
pub mod lower;

pub use types::{IrType, GenericParam, StructDef};
pub use instruction::{Instruction, Value, BinaryOp, CompareOp};
pub use module::{Module, Function, BasicBlock, HttpRoute, WsRoute, Global, TraitDef, TraitMethodDef, ImplDef, TypeAlias};
pub use lower::lower_program;
