//! IR Type System
//!
//! Low-level types for IR representation.

use std::fmt;

/// IR Types (lower level than MendesType)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IrType {
    /// Void / Unit
    Void,
    /// 64-bit integer
    I64,
    /// 64-bit float
    F64,
    /// Boolean (i1 in LLVM)
    Bool,
    /// Pointer to string (i8*)
    String,
    /// Pointer to type
    Ptr(Box<IrType>),
    /// Fixed-size array
    Array(Box<IrType>, usize),
    /// Struct by name
    Struct(String),
    /// Function
    Function {
        params: Vec<IrType>,
        ret: Box<IrType>,
    },
    /// Future (for async) - internally a pointer to state machine
    Future(Box<IrType>),
    /// Tuple type
    Tuple(Vec<IrType>),
    /// Range type (for iterating)
    Range(Box<IrType>),
}

impl IrType {
    /// Returns the size in bytes (approximate, for allocation)
    pub fn size_bytes(&self) -> usize {
        match self {
            IrType::Void => 0,
            IrType::Bool => 1,
            IrType::I64 => 8,
            IrType::F64 => 8,
            IrType::String => 16, // ptr + len
            IrType::Ptr(_) => 8,
            IrType::Array(elem, count) => elem.size_bytes() * count,
            IrType::Struct(_) => 0, // Needs lookup in struct table
            IrType::Function { .. } => 8, // pointer to function
            IrType::Future(_) => 8, // pointer to state machine
            IrType::Tuple(elems) => elems.iter().map(|e| e.size_bytes()).sum(),
            IrType::Range(_) => 24, // start + end + inclusive flag
        }
    }

    /// Checks if it is a primitive type
    pub fn is_primitive(&self) -> bool {
        matches!(self, IrType::Void | IrType::Bool | IrType::I64 | IrType::F64)
    }

    /// Checks if it is a pointer
    pub fn is_pointer(&self) -> bool {
        matches!(self, IrType::Ptr(_) | IrType::String | IrType::Future(_))
    }

    /// Converts from MendesType to IrType
    pub fn from_mendes_type(ty: &mendes_parser::Type) -> Self {
        match ty {
            mendes_parser::Type::Int => IrType::I64,
            mendes_parser::Type::Float => IrType::F64,
            mendes_parser::Type::Bool => IrType::Bool,
            mendes_parser::Type::String => IrType::String,
            mendes_parser::Type::Named(name) => IrType::Struct(name.clone()),
            mendes_parser::Type::Generic { name, args } => {
                match name.as_str() {
                    "Result" | "Option" => {
                        // Result<T, E> and Option<T> are represented as tagged unions
                        // For simplicity, we use a struct
                        IrType::Struct(format!("{}_{}", name,
                            args.iter().map(|a| format!("{:?}", a)).collect::<Vec<_>>().join("_")))
                    }
                    _ => IrType::Struct(name.clone()),
                }
            }
            mendes_parser::Type::Ref(inner) => {
                IrType::Ptr(Box::new(Self::from_mendes_type(inner)))
            }
            mendes_parser::Type::MutRef(inner) => {
                IrType::Ptr(Box::new(Self::from_mendes_type(inner)))
            }
            mendes_parser::Type::Array(inner) => {
                // Dynamic arrays are pointers + size
                IrType::Ptr(Box::new(Self::from_mendes_type(inner)))
            }
            mendes_parser::Type::Tuple(types) => {
                IrType::Tuple(types.iter().map(Self::from_mendes_type).collect())
            }
            mendes_parser::Type::Function { params, return_type } => {
                IrType::Function {
                    params: params.iter().map(Self::from_mendes_type).collect(),
                    ret: Box::new(Self::from_mendes_type(return_type)),
                }
            }
        }
    }
}

impl fmt::Display for IrType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IrType::Void => write!(f, "void"),
            IrType::I64 => write!(f, "i64"),
            IrType::F64 => write!(f, "f64"),
            IrType::Bool => write!(f, "i1"),
            IrType::String => write!(f, "string"),
            IrType::Ptr(inner) => write!(f, "*{}", inner),
            IrType::Array(elem, size) => write!(f, "[{} x {}]", size, elem),
            IrType::Struct(name) => write!(f, "%{}", name),
            IrType::Function { params, ret } => {
                write!(f, "fn(")?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", p)?;
                }
                write!(f, ") -> {}", ret)
            }
            IrType::Future(inner) => write!(f, "future<{}>", inner),
            IrType::Tuple(elems) => {
                write!(f, "(")?;
                for (i, e) in elems.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", e)?;
                }
                write!(f, ")")
            }
            IrType::Range(inner) => write!(f, "range<{}>", inner),
        }
    }
}

/// Generic parameter in IR
#[derive(Debug, Clone)]
pub struct GenericParam {
    /// Parameter name (e.g., "T")
    pub name: String,
    /// Trait bounds (e.g., ["Show", "Clone"])
    pub bounds: Vec<String>,
}

impl GenericParam {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            bounds: Vec::new(),
        }
    }

    pub fn with_bounds(mut self, bounds: Vec<String>) -> Self {
        self.bounds = bounds;
        self
    }
}

/// Struct definition in IR
#[derive(Debug, Clone)]
pub struct StructDef {
    pub name: String,
    /// Generic type parameters
    pub generic_params: Vec<GenericParam>,
    pub fields: Vec<(String, IrType)>,
    /// Method names (the actual methods are stored as separate functions)
    pub methods: Vec<String>,
}

impl StructDef {
    pub fn new(name: String) -> Self {
        Self { name, generic_params: Vec::new(), fields: Vec::new(), methods: Vec::new() }
    }

    pub fn with_generics(mut self, params: Vec<GenericParam>) -> Self {
        self.generic_params = params;
        self
    }

    pub fn add_generic_param(&mut self, param: GenericParam) {
        self.generic_params.push(param);
    }

    pub fn add_field(&mut self, name: String, ty: IrType) {
        self.fields.push((name, ty));
    }

    pub fn add_method(&mut self, name: String) {
        self.methods.push(name);
    }

    pub fn field_index(&self, name: &str) -> Option<usize> {
        self.fields.iter().position(|(n, _)| n == name)
    }

    pub fn field_type(&self, name: &str) -> Option<&IrType> {
        self.fields.iter().find(|(n, _)| n == name).map(|(_, t)| t)
    }

    pub fn has_method(&self, name: &str) -> bool {
        self.methods.iter().any(|m| m == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ir_type_display() {
        assert_eq!(IrType::I64.to_string(), "i64");
        assert_eq!(IrType::Ptr(Box::new(IrType::I64)).to_string(), "*i64");
        assert_eq!(IrType::Array(Box::new(IrType::I64), 10).to_string(), "[10 x i64]");
    }

    #[test]
    fn test_struct_def() {
        let mut s = StructDef::new("User".to_string());
        s.add_field("id".to_string(), IrType::I64);
        s.add_field("name".to_string(), IrType::String);

        assert_eq!(s.field_index("id"), Some(0));
        assert_eq!(s.field_index("name"), Some(1));
        assert_eq!(s.field_type("id"), Some(&IrType::I64));
    }
}
