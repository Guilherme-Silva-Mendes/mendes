//! Type system for the Mendes language

use std::collections::HashMap;
use std::fmt;

/// Unique ID of a type
pub type TypeId = u32;

/// Types of the Mendes language
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MendesType {
    /// Unknown type (error or pending inference)
    Unknown,
    /// Any type (accepts any type, for built-ins)
    Any,
    /// Unit type (void)
    Unit,
    /// Integer
    Int,
    /// Float
    Float,
    /// Boolean
    Bool,
    /// String
    String,
    /// Named type (struct, enum)
    Named(std::string::String),
    /// Generic type: Result<T, E>, Option<T>
    Generic {
        name: std::string::String,
        args: Vec<MendesType>,
    },
    /// Immutable reference
    Ref(Box<MendesType>),
    /// Mutable reference
    MutRef(Box<MendesType>),
    /// Array
    Array(Box<MendesType>),
    /// Function
    Function {
        params: Vec<MendesType>,
        ret: Box<MendesType>,
    },
    /// Future (result of async function)
    Future(Box<MendesType>),
    /// Tuple
    Tuple(Vec<MendesType>),
    /// Range (for iteration)
    Range(Box<MendesType>),
}

impl MendesType {
    /// Checks if the type is primitive (copy by default)
    pub fn is_copy(&self) -> bool {
        matches!(self, MendesType::Int | MendesType::Float | MendesType::Bool | MendesType::Unit)
    }

    /// Checks if it's a reference
    pub fn is_ref(&self) -> bool {
        matches!(self, MendesType::Ref(_) | MendesType::MutRef(_))
    }

    /// Returns the inner type of a reference
    pub fn inner_type(&self) -> Option<&MendesType> {
        match self {
            MendesType::Ref(inner) | MendesType::MutRef(inner) => Some(inner),
            _ => None,
        }
    }

    /// Checks if two types are compatible
    pub fn is_compatible_with(&self, other: &MendesType) -> bool {
        match (self, other) {
            // Unknown and Any are compatible with any type
            (MendesType::Unknown, _) | (_, MendesType::Unknown) => true,
            (MendesType::Any, _) | (_, MendesType::Any) => true,
            (MendesType::Int, MendesType::Int) => true,
            (MendesType::Float, MendesType::Float) => true,
            (MendesType::Bool, MendesType::Bool) => true,
            (MendesType::String, MendesType::String) => true,
            (MendesType::Unit, MendesType::Unit) => true,
            (MendesType::Named(a), MendesType::Named(b)) => a == b,
            (MendesType::Array(a), MendesType::Array(b)) => a.is_compatible_with(b),
            (MendesType::Ref(a), MendesType::Ref(b)) => a.is_compatible_with(b),
            (MendesType::MutRef(a), MendesType::MutRef(b)) => a.is_compatible_with(b),
            (MendesType::Generic { name: n1, args: a1 }, MendesType::Generic { name: n2, args: a2 }) => {
                n1 == n2 && a1.len() == a2.len() && a1.iter().zip(a2).all(|(t1, t2)| t1.is_compatible_with(t2))
            }
            (MendesType::Tuple(a), MendesType::Tuple(b)) => {
                a.len() == b.len() && a.iter().zip(b).all(|(t1, t2)| t1.is_compatible_with(t2))
            }
            (MendesType::Range(a), MendesType::Range(b)) => a.is_compatible_with(b),
            (MendesType::Function { params: p1, ret: r1 }, MendesType::Function { params: p2, ret: r2 }) => {
                p1.len() == p2.len() &&
                p1.iter().zip(p2).all(|(t1, t2)| t1.is_compatible_with(t2)) &&
                r1.is_compatible_with(r2)
            }
            // Closure type (Generic "Fn") is compatible with Function type
            (MendesType::Generic { name, args }, MendesType::Function { params, ret }) if name == "Fn" => {
                // Generic Fn stores [param_types..., ret_type]
                if args.len() != params.len() + 1 {
                    return false;
                }
                // Check param types
                let params_match = params.iter().zip(args.iter())
                    .all(|(p, a)| p.is_compatible_with(a));
                // Check return type (last arg in Fn<...>)
                let ret_matches = args.last()
                    .map(|r| ret.is_compatible_with(r))
                    .unwrap_or(true);
                params_match && ret_matches
            }
            (MendesType::Function { params, ret }, MendesType::Generic { name, args }) if name == "Fn" => {
                // Same as above, just swapped
                if args.len() != params.len() + 1 {
                    return false;
                }
                let params_match = params.iter().zip(args.iter())
                    .all(|(p, a)| p.is_compatible_with(a));
                let ret_matches = args.last()
                    .map(|r| ret.is_compatible_with(r))
                    .unwrap_or(true);
                params_match && ret_matches
            }
            _ => false,
        }
    }

    /// Converts from AST Type to MendesType
    pub fn from_ast(ast_type: &mendes_parser::Type) -> Self {
        match ast_type {
            mendes_parser::Type::Int => MendesType::Int,
            mendes_parser::Type::Float => MendesType::Float,
            mendes_parser::Type::Bool => MendesType::Bool,
            mendes_parser::Type::String => MendesType::String,
            mendes_parser::Type::Named(name) => MendesType::Named(name.clone()),
            mendes_parser::Type::Generic { name, args } => MendesType::Generic {
                name: name.clone(),
                args: args.iter().map(MendesType::from_ast).collect(),
            },
            mendes_parser::Type::Ref(inner) => MendesType::Ref(Box::new(MendesType::from_ast(inner))),
            mendes_parser::Type::MutRef(inner) => MendesType::MutRef(Box::new(MendesType::from_ast(inner))),
            mendes_parser::Type::Array(inner) => MendesType::Array(Box::new(MendesType::from_ast(inner))),
            mendes_parser::Type::Tuple(types) => {
                MendesType::Tuple(types.iter().map(MendesType::from_ast).collect())
            }
            mendes_parser::Type::Function { params, return_type } => {
                MendesType::Function {
                    params: params.iter().map(MendesType::from_ast).collect(),
                    ret: Box::new(MendesType::from_ast(return_type)),
                }
            }
        }
    }
}

impl fmt::Display for MendesType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MendesType::Unknown => write!(f, "?"),
            MendesType::Any => write!(f, "any"),
            MendesType::Unit => write!(f, "()"),
            MendesType::Int => write!(f, "int"),
            MendesType::Float => write!(f, "float"),
            MendesType::Bool => write!(f, "bool"),
            MendesType::String => write!(f, "string"),
            MendesType::Named(name) => write!(f, "{}", name),
            MendesType::Generic { name, args } => {
                write!(f, "{}<", name)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ">")
            }
            MendesType::Ref(inner) => write!(f, "&{}", inner),
            MendesType::MutRef(inner) => write!(f, "&mut {}", inner),
            MendesType::Array(inner) => write!(f, "[{}]", inner),
            MendesType::Function { params, ret } => {
                write!(f, "fn(")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", param)?;
                }
                write!(f, ") -> {}", ret)
            }
            MendesType::Future(inner) => write!(f, "Future<{}>", inner),
            MendesType::Tuple(types) => {
                write!(f, "(")?;
                for (i, t) in types.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", t)?;
                }
                write!(f, ")")
            }
            MendesType::Range(inner) => write!(f, "Range<{}>", inner),
        }
    }
}

/// Definition of a struct
#[derive(Debug, Clone)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<(String, MendesType)>,
    /// Methods: (name, params, return_type, is_async)
    pub methods: Vec<(String, Vec<(String, MendesType)>, MendesType, bool)>,
    pub is_copy: bool,
}

/// Registry of user-defined types
#[derive(Debug, Default)]
pub struct TypeRegistry {
    structs: HashMap<String, StructDef>,
    /// Currently active generic type parameters (e.g., T, U in struct<T, U>)
    generic_params: std::collections::HashSet<String>,
}

impl TypeRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a struct
    pub fn register_struct(&mut self, def: StructDef) {
        self.structs.insert(def.name.clone(), def);
    }

    /// Looks up a struct by name
    pub fn get_struct(&self, name: &str) -> Option<&StructDef> {
        self.structs.get(name)
    }

    /// Registers a generic type parameter (e.g., T in struct Pair<T>)
    pub fn register_generic_param(&mut self, name: &str) {
        self.generic_params.insert(name.to_string());
    }

    /// Unregisters a generic type parameter
    pub fn unregister_generic_param(&mut self, name: &str) {
        self.generic_params.remove(name);
    }

    /// Checks if a name is a registered generic parameter
    pub fn is_generic_param(&self, name: &str) -> bool {
        self.generic_params.contains(name)
    }

    /// Checks if a type exists
    pub fn type_exists(&self, ty: &MendesType) -> bool {
        match ty {
            MendesType::Int | MendesType::Float | MendesType::Bool |
            MendesType::String | MendesType::Unit | MendesType::Unknown | MendesType::Any => true,
            MendesType::Named(name) => {
                self.structs.contains_key(name) ||
                self.generic_params.contains(name) ||
                name == "HttpError" || name == "Response" // Built-in types
            },
            MendesType::Generic { name, args } => {
                (name == "Result" || name == "Option") &&
                args.iter().all(|a| self.type_exists(a))
            }
            MendesType::Ref(inner) | MendesType::MutRef(inner) | MendesType::Array(inner) => {
                self.type_exists(inner)
            }
            MendesType::Function { params, ret } => {
                params.iter().all(|p| self.type_exists(p)) && self.type_exists(ret)
            }
            MendesType::Future(inner) => self.type_exists(inner),
            MendesType::Tuple(types) => types.iter().all(|t| self.type_exists(t)),
            MendesType::Range(inner) => self.type_exists(inner),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_compatibility() {
        assert!(MendesType::Int.is_compatible_with(&MendesType::Int));
        assert!(!MendesType::Int.is_compatible_with(&MendesType::String));
        assert!(MendesType::Unknown.is_compatible_with(&MendesType::Int));
    }

    #[test]
    fn test_type_display() {
        assert_eq!(MendesType::Int.to_string(), "int");
        assert_eq!(
            MendesType::Generic {
                name: "Result".to_string(),
                args: vec![MendesType::Int, MendesType::String],
            }
            .to_string(),
            "Result<int, string>"
        );
    }

    #[test]
    fn test_is_copy() {
        assert!(MendesType::Int.is_copy());
        assert!(MendesType::Bool.is_copy());
        assert!(!MendesType::String.is_copy());
        assert!(!MendesType::Named("User".to_string()).is_copy());
    }
}
