//! Ownership and Borrow Checker for the Mendes language
//!
//! Implements Rust-like ownership rules:
//! - Each value has a single owner
//! - Move by default (except Copy types)
//! - Immutable borrow (&) or mutable borrow (&mut)
//! - One mutability at a time
//! - References cannot cross await

use crate::types::MendesType;
use mendes_error::{Diagnostic, Diagnostics, ErrorCode, Span};
use std::collections::HashMap;

/// State of a variable with respect to ownership
#[derive(Debug, Clone, PartialEq)]
pub enum OwnershipState {
    /// Variable owns the value and is available
    Owned,
    /// Value was moved elsewhere
    Moved { moved_at: Span },
    /// Value is immutably borrowed
    Borrowed { borrow_count: u32, borrowed_at: Vec<Span> },
    /// Value is mutably borrowed
    MutBorrowed { borrowed_at: Span },
}

/// Ownership information for a variable
#[derive(Debug, Clone)]
pub struct OwnershipInfo {
    /// Variable name
    pub name: String,
    /// Variable type
    pub ty: MendesType,
    /// Current state
    pub state: OwnershipState,
    /// Whether the variable is mutable
    pub is_mutable: bool,
    /// Where it was defined
    pub defined_at: Span,
}

impl OwnershipInfo {
    pub fn new(name: String, ty: MendesType, is_mutable: bool, defined_at: Span) -> Self {
        Self {
            name,
            ty,
            state: OwnershipState::Owned,
            is_mutable,
            defined_at,
        }
    }

    /// Checks if it can be used (read)
    pub fn can_use(&self) -> bool {
        !matches!(self.state, OwnershipState::Moved { .. })
    }

    /// Checks if it can be moved
    pub fn can_move(&self) -> bool {
        matches!(self.state, OwnershipState::Owned)
    }

    /// Checks if it can be immutably borrowed
    pub fn can_borrow(&self) -> bool {
        matches!(self.state, OwnershipState::Owned | OwnershipState::Borrowed { .. })
    }

    /// Checks if it can be mutably borrowed
    pub fn can_borrow_mut(&self) -> bool {
        matches!(self.state, OwnershipState::Owned) && self.is_mutable
    }
}

/// Ownership scope
#[derive(Debug, Default)]
pub struct OwnershipScope {
    variables: HashMap<String, OwnershipInfo>,
    /// Active borrows in this scope
    active_borrows: Vec<(String, bool, Span)>, // (var_name, is_mut, span)
}

impl OwnershipScope {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Ownership Checker
#[derive(Debug)]
pub struct OwnershipChecker {
    /// Scope stack
    scopes: Vec<OwnershipScope>,
    /// Diagnostics
    diagnostics: Diagnostics,
    /// Whether we are inside an async context
    in_async_context: bool,
    /// Active borrows before an await (to check if they cross)
    borrows_before_await: Vec<(String, Span)>,
}

impl OwnershipChecker {
    pub fn new() -> Self {
        Self {
            scopes: vec![OwnershipScope::new()],
            diagnostics: Diagnostics::new(),
            in_async_context: false,
            borrows_before_await: Vec::new(),
        }
    }

    pub fn take_diagnostics(&mut self) -> Diagnostics {
        std::mem::take(&mut self.diagnostics)
    }

    /// Enters a new scope
    pub fn push_scope(&mut self) {
        self.scopes.push(OwnershipScope::new());
    }

    /// Exits the current scope
    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    /// Defines a new variable
    pub fn define(&mut self, name: String, ty: MendesType, is_mutable: bool, span: Span) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.variables.insert(
                name.clone(),
                OwnershipInfo::new(name, ty, is_mutable, span),
            );
        }
    }

    /// Looks up ownership information for a variable
    pub fn lookup(&self, name: &str) -> Option<&OwnershipInfo> {
        for scope in self.scopes.iter().rev() {
            if let Some(info) = scope.variables.get(name) {
                return Some(info);
            }
        }
        None
    }

    /// Mutable lookup
    fn lookup_mut(&mut self, name: &str) -> Option<&mut OwnershipInfo> {
        for scope in self.scopes.iter_mut().rev() {
            if scope.variables.contains_key(name) {
                return scope.variables.get_mut(name);
            }
        }
        None
    }

    /// Marks variable as moved
    pub fn mark_moved(&mut self, name: &str, moved_at: Span) {
        if let Some(info) = self.lookup_mut(name) {
            // Copy types are not moved
            if info.ty.is_copy() {
                return;
            }

            info.state = OwnershipState::Moved { moved_at };
        }
    }

    /// Registers an immutable borrow
    pub fn borrow(&mut self, name: &str, borrow_span: Span) -> Result<(), Diagnostic> {
        if let Some(info) = self.lookup_mut(name) {
            match &info.state {
                OwnershipState::Owned => {
                    info.state = OwnershipState::Borrowed {
                        borrow_count: 1,
                        borrowed_at: vec![borrow_span],
                    };

                    if self.in_async_context {
                        self.borrows_before_await.push((name.to_string(), borrow_span));
                    }

                    Ok(())
                }
                OwnershipState::Borrowed { borrow_count, borrowed_at } => {
                    let mut new_borrowed_at = borrowed_at.clone();
                    new_borrowed_at.push(borrow_span);
                    info.state = OwnershipState::Borrowed {
                        borrow_count: borrow_count + 1,
                        borrowed_at: new_borrowed_at,
                    };

                    if self.in_async_context {
                        self.borrows_before_await.push((name.to_string(), borrow_span));
                    }

                    Ok(())
                }
                OwnershipState::MutBorrowed { borrowed_at } => {
                    Err(Diagnostic::error(format!("cannot borrow `{}` while it is mutably borrowed", name))
                        .with_code(ErrorCode::MUT_BORROW_CONFLICT)
                        .with_label(borrow_span, "attempting to borrow here")
                        .with_secondary_label(*borrowed_at, "mutable borrow active here"))
                }
                OwnershipState::Moved { moved_at } => {
                    Err(Diagnostic::error(format!("use of `{}` after move", name))
                        .with_code(ErrorCode::USE_AFTER_MOVE)
                        .with_label(borrow_span, "use after move")
                        .with_secondary_label(*moved_at, "value moved here"))
                }
            }
        } else {
            Ok(()) // Variable not tracked
        }
    }

    /// Registers a mutable borrow
    pub fn borrow_mut(&mut self, name: &str, borrow_span: Span) -> Result<(), Diagnostic> {
        if let Some(info) = self.lookup_mut(name) {
            if !info.is_mutable {
                return Err(Diagnostic::error(format!("cannot borrow `{}` mutably - not mutable", name))
                    .with_code(ErrorCode::MUT_BORROW_CONFLICT)
                    .with_label(borrow_span, "attempting to borrow mutably")
                    .with_secondary_label(info.defined_at, "defined as immutable here")
                    .with_help("add `mut` to the declaration: `let mut {}`".to_string()));
            }

            match &info.state {
                OwnershipState::Owned => {
                    info.state = OwnershipState::MutBorrowed { borrowed_at: borrow_span };

                    if self.in_async_context {
                        self.borrows_before_await.push((name.to_string(), borrow_span));
                    }

                    Ok(())
                }
                OwnershipState::Borrowed { borrowed_at, .. } => {
                    Err(Diagnostic::error(format!("cannot borrow `{}` mutably while it is borrowed", name))
                        .with_code(ErrorCode::MUT_BORROW_CONFLICT)
                        .with_label(borrow_span, "attempting to borrow mutably")
                        .with_secondary_label(borrowed_at[0], "immutable borrow active here"))
                }
                OwnershipState::MutBorrowed { borrowed_at } => {
                    Err(Diagnostic::error(format!("cannot borrow `{}` mutably more than once", name))
                        .with_code(ErrorCode::MUT_BORROW_CONFLICT)
                        .with_label(borrow_span, "second mutable borrow")
                        .with_secondary_label(*borrowed_at, "first mutable borrow here"))
                }
                OwnershipState::Moved { moved_at } => {
                    Err(Diagnostic::error(format!("use of `{}` after move", name))
                        .with_code(ErrorCode::USE_AFTER_MOVE)
                        .with_label(borrow_span, "use after move")
                        .with_secondary_label(*moved_at, "value moved here"))
                }
            }
        } else {
            Ok(())
        }
    }

    /// Checks variable usage
    pub fn check_use(&mut self, name: &str, use_span: Span) -> Result<(), Diagnostic> {
        if let Some(info) = self.lookup(name) {
            if let OwnershipState::Moved { moved_at } = info.state {
                return Err(Diagnostic::error(format!("use of `{}` after move", name))
                    .with_code(ErrorCode::USE_AFTER_MOVE)
                    .with_label(use_span, "use after move")
                    .with_secondary_label(moved_at, "value moved here")
                    .with_help("consider cloning the value or using a reference".to_string()));
            }
        }
        Ok(())
    }

    /// Enters async context
    pub fn enter_async(&mut self) {
        self.in_async_context = true;
        self.borrows_before_await.clear();
    }

    /// Exits async context
    pub fn exit_async(&mut self) {
        self.in_async_context = false;
        self.borrows_before_await.clear();
    }

    /// Checks borrows crossing await
    pub fn check_await(&mut self, await_span: Span) {
        for (var_name, borrow_span) in &self.borrows_before_await {
            self.diagnostics.push(
                Diagnostic::error(format!("reference to `{}` cannot cross await", var_name))
                    .with_code(ErrorCode::BORROW_ACROSS_AWAIT)
                    .with_label(*borrow_span, "reference created here")
                    .with_secondary_label(await_span, "await happens here")
                    .with_help("copy the value before await or use owned type".to_string())
            );
        }
    }

    /// Releases all borrows from the current scope
    pub fn release_borrows(&mut self) {
        if let Some(scope) = self.scopes.last_mut() {
            for info in scope.variables.values_mut() {
                match &info.state {
                    OwnershipState::Borrowed { .. } | OwnershipState::MutBorrowed { .. } => {
                        info.state = OwnershipState::Owned;
                    }
                    _ => {}
                }
            }
        }
    }
}

impl Default for OwnershipChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mendes_error::span::Position;

    fn make_span(line: u32) -> Span {
        Span::new(
            Position::new(line, 1, 0),
            Position::new(line, 10, 10),
            0,
        )
    }

    #[test]
    fn test_use_after_move() {
        let mut checker = OwnershipChecker::new();

        checker.define("x".to_string(), MendesType::String, false, make_span(1));
        checker.mark_moved("x", make_span(2));

        let result = checker.check_use("x", make_span(3));
        assert!(result.is_err());
    }

    #[test]
    fn test_copy_types_not_moved() {
        let mut checker = OwnershipChecker::new();

        checker.define("x".to_string(), MendesType::Int, false, make_span(1));
        checker.mark_moved("x", make_span(2));

        // Int is Copy, so it's not moved
        let result = checker.check_use("x", make_span(3));
        assert!(result.is_ok());
    }

    #[test]
    fn test_borrow_conflict() {
        let mut checker = OwnershipChecker::new();

        checker.define("x".to_string(), MendesType::String, true, make_span(1));

        // First mutable borrow
        assert!(checker.borrow_mut("x", make_span(2)).is_ok());

        // Second borrow should fail
        assert!(checker.borrow("x", make_span(3)).is_err());
    }

    #[test]
    fn test_multiple_immutable_borrows() {
        let mut checker = OwnershipChecker::new();

        checker.define("x".to_string(), MendesType::String, false, make_span(1));

        // Multiple immutable borrows are OK
        assert!(checker.borrow("x", make_span(2)).is_ok());
        assert!(checker.borrow("x", make_span(3)).is_ok());
        assert!(checker.borrow("x", make_span(4)).is_ok());
    }
}
