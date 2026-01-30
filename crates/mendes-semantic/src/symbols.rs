//! Symbol table for the Mendes language

use crate::types::MendesType;
use mendes_error::Span;
use std::collections::HashMap;

/// Symbol type
#[derive(Debug, Clone)]
pub enum SymbolKind {
    /// Local variable
    Variable,
    /// Function parameter
    Parameter,
    /// Function
    Function {
        /// Generic type parameters (e.g., ["T", "U"])
        generic_params: Vec<String>,
        params: Vec<(String, MendesType)>,
        return_type: MendesType,
        is_async: bool,
    },
    /// Struct
    Struct {
        fields: Vec<(String, MendesType)>,
        /// Methods: (name, params, return_type, is_async)
        methods: Vec<(String, Vec<(String, MendesType)>, MendesType, bool)>,
        is_copy: bool,
    },
    /// Enum
    Enum {
        /// Variants: (name, associated types)
        variants: Vec<(String, Vec<MendesType>)>,
    },
    /// Struct field
    Field,
    /// Database connection
    Database {
        db_type: String,
        pool_size: u32,
    },
    /// Middleware
    Middleware,
    /// Trait definition
    Trait,
    /// Type alias
    TypeAlias,
}

/// A symbol in the table
#[derive(Debug, Clone)]
pub struct Symbol {
    /// Symbol name
    pub name: String,
    /// Symbol type
    pub kind: SymbolKind,
    /// Value type
    pub ty: MendesType,
    /// Whether it's mutable
    pub mutable: bool,
    /// Where it was defined
    pub defined_at: Option<Span>,
}

impl Symbol {
    pub fn new(name: String, ty: MendesType, kind: SymbolKind, span: Span) -> Self {
        Self {
            name,
            kind,
            ty,
            mutable: false,
            defined_at: Some(span),
        }
    }

    pub fn variable(name: String, ty: MendesType, mutable: bool, span: Span) -> Self {
        Self {
            name,
            kind: SymbolKind::Variable,
            ty,
            mutable,
            defined_at: Some(span),
        }
    }

    pub fn parameter(name: String, ty: MendesType, span: Span) -> Self {
        Self {
            name,
            kind: SymbolKind::Parameter,
            ty,
            mutable: false,
            defined_at: Some(span),
        }
    }
}

/// Symbol scope
#[derive(Debug, Default)]
pub struct Scope {
    symbols: HashMap<String, Symbol>,
}

impl Scope {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn define(&mut self, symbol: Symbol) -> Option<Symbol> {
        self.symbols.insert(symbol.name.clone(), symbol)
    }

    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name)
    }

    pub fn lookup_mut(&mut self, name: &str) -> Option<&mut Symbol> {
        self.symbols.get_mut(name)
    }
}

/// Symbol table with support for nested scopes
#[derive(Debug)]
pub struct SymbolTable {
    /// Scope stack (the last one is the current scope)
    scopes: Vec<Scope>,
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            scopes: vec![Scope::new()], // Global scope
        }
    }

    /// Enters a new scope
    pub fn push_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    /// Exits the current scope
    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    /// Defines a symbol in the current scope
    pub fn define(&mut self, symbol: Symbol) -> Option<Symbol> {
        if let Some(scope) = self.scopes.last_mut() {
            scope.define(symbol)
        } else {
            None
        }
    }

    /// Looks up a symbol in all scopes (from innermost to outermost)
    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        for scope in self.scopes.iter().rev() {
            if let Some(symbol) = scope.lookup(name) {
                return Some(symbol);
            }
        }
        None
    }

    /// Looks up a mutable symbol
    pub fn lookup_mut(&mut self, name: &str) -> Option<&mut Symbol> {
        for scope in self.scopes.iter_mut().rev() {
            if scope.lookup(name).is_some() {
                return scope.lookup_mut(name);
            }
        }
        None
    }

    /// Looks up only in the current scope
    pub fn lookup_current_scope(&self, name: &str) -> Option<&Symbol> {
        self.scopes.last()?.lookup(name)
    }

    /// Checks if a symbol exists in the current scope
    pub fn is_defined_in_current_scope(&self, name: &str) -> bool {
        self.lookup_current_scope(name).is_some()
    }

    /// Returns the depth of the current scope
    pub fn depth(&self) -> usize {
        self.scopes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_table_scopes() {
        let mut table = SymbolTable::new();

        // Define in the global scope
        table.define(Symbol {
            name: "x".to_string(),
            kind: SymbolKind::Variable,
            ty: MendesType::Int,
            mutable: false,
            defined_at: None,
        });

        assert!(table.lookup("x").is_some());

        // Enter a new scope
        table.push_scope();

        // x is still visible
        assert!(table.lookup("x").is_some());

        // Define y in the inner scope
        table.define(Symbol {
            name: "y".to_string(),
            kind: SymbolKind::Variable,
            ty: MendesType::Int,
            mutable: false,
            defined_at: None,
        });

        assert!(table.lookup("y").is_some());

        // Exit the scope
        table.pop_scope();

        // y is no longer visible
        assert!(table.lookup("y").is_none());
        // x is still visible
        assert!(table.lookup("x").is_some());
    }

    #[test]
    fn test_shadowing() {
        let mut table = SymbolTable::new();

        table.define(Symbol {
            name: "x".to_string(),
            kind: SymbolKind::Variable,
            ty: MendesType::Int,
            mutable: false,
            defined_at: None,
        });

        table.push_scope();

        // Shadowing: same name, different type
        table.define(Symbol {
            name: "x".to_string(),
            kind: SymbolKind::Variable,
            ty: MendesType::String,
            mutable: false,
            defined_at: None,
        });

        // In the inner scope, x is String
        assert_eq!(table.lookup("x").unwrap().ty, MendesType::String);

        table.pop_scope();

        // In the outer scope, x goes back to Int
        assert_eq!(table.lookup("x").unwrap().ty, MendesType::Int);
    }
}
