//! mendes-codegen - Code generation for the Mendes language
//!
//! Supports multiple backends:
//! - **C Backend**: Generates portable C code (default)
//! - **LLVM Backend**: Generates LLVM IR directly (requires LLVM installed)
//!
//! # Example
//!
//! ```rust,ignore
//! use mendes_codegen::{CodeGen, CBackend};
//! use mendes_ir::Module;
//!
//! let module: Module = /* ... */;
//! let backend = CBackend::new();
//! let c_code = backend.generate(&module);
//! ```

pub mod c_backend;
pub mod rust_backend;

#[cfg(feature = "llvm")]
pub mod llvm_backend;

pub use c_backend::CBackend;
pub use rust_backend::RustBackend;

#[cfg(feature = "llvm")]
pub use llvm_backend::{LlvmBackend, LlvmCodeGen};

/// Trait for code generation backends
pub trait CodeGen {
    /// Backend output type
    type Output;

    /// Generates code from the IR module
    fn generate(&self, module: &mendes_ir::Module) -> Self::Output;
}

/// Compilation options
#[derive(Debug, Clone)]
pub struct CompileOptions {
    /// Output file name
    pub output: String,
    /// Optimization level (0-3)
    pub opt_level: u8,
    /// Include debug information
    pub debug_info: bool,
    /// Target triple (e.g., x86_64-pc-windows-msvc)
    pub target: Option<String>,
}

impl Default for CompileOptions {
    fn default() -> Self {
        Self {
            output: "output".to_string(),
            opt_level: 0,
            debug_info: true,
            target: None,
        }
    }
}
