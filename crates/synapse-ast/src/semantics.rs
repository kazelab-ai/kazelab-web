//! Static Semantic Analyzer, Borrow Scope Checker, and Type Well-Formedness Linter.
//! Validates AST structure against Rust memory model invariants prior to compilation.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SemanticErrorKind {
    UseAfterMove,
    MultipleMutableBorrows,
    MutableBorrowWithSharedBorrow,
    UninitializedVariableRead,
    UnreachableCodeBranch,
    UnusedBinding,
    TypeMismatch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticDiagnostic {
    pub kind: SemanticErrorKind,
    pub symbol_name: String,
    pub span: (usize, usize),
    pub message: String,
    pub suggested_fix: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OwnershipState {
    Uninitialized,
    Owned,
    Moved,
    SharedBorrowed(usize), // Active shared borrow count
    MutablyBorrowed,
}

#[derive(Debug, Clone)]
pub struct ScopeFrame {
    pub bindings: HashMap<String, OwnershipState>,
    pub declared_types: HashMap<String, String>,
}

impl ScopeFrame {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            declared_types: HashMap::new(),
        }
    }
}

pub struct SemanticAnalyzer {
    scopes: Vec<ScopeFrame>,
    diagnostics: Vec<SemanticDiagnostic>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            scopes: vec![ScopeFrame::new()],
            diagnostics: Vec::new(),
        }
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(ScopeFrame::new());
    }

    pub fn exit_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    pub fn declare_symbol(&mut self, name: &str, type_str: &str, initial_state: OwnershipState) {
        if let Some(current_scope) = self.scopes.last_mut() {
            current_scope.bindings.insert(name.to_string(), initial_state);
            current_scope.declared_types.insert(name.to_string(), type_str.to_string());
        }
    }

    pub fn record_move(&mut self, name: &str, span: (usize, usize)) {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(state) = scope.bindings.get_mut(name) {
                match *state {
                    OwnershipState::Moved => {
                        self.diagnostics.push(SemanticDiagnostic {
                            kind: SemanticErrorKind::UseAfterMove,
                            symbol_name: name.to_string(),
                            span,
                            message: format!("Attempted to move value `{}` which was previously moved.", name),
                            suggested_fix: Some(format!("Consider cloning `{}`: `{}.clone()`", name, name)),
                        });
                    }
                    OwnershipState::SharedBorrowed(_) => {
                        self.diagnostics.push(SemanticDiagnostic {
                            kind: SemanticErrorKind::UseAfterMove,
                            symbol_name: name.to_string(),
                            span,
                            message: format!("Cannot move `{}` while it is actively borrowed.", name),
                            suggested_fix: None,
                        });
                    }
                    OwnershipState::MutablyBorrowed => {
                        self.diagnostics.push(SemanticDiagnostic {
                            kind: SemanticErrorKind::UseAfterMove,
                            symbol_name: name.to_string(),
                            span,
                            message: format!("Cannot move `{}` while mutable borrow is outstanding.", name),
                            suggested_fix: None,
                        });
                    }
                    OwnershipState::Uninitialized => {
                        self.diagnostics.push(SemanticDiagnostic {
                            kind: SemanticErrorKind::UninitializedVariableRead,
                            symbol_name: name.to_string(),
                            span,
                            message: format!("Variable `{}` used before assignment.", name),
                            suggested_fix: Some(format!("Initialize `{}` prior to use.", name)),
                        });
                    }
                    OwnershipState::Owned => {
                        *state = OwnershipState::Moved;
                    }
                }
                return;
            }
        }
    }

    pub fn record_borrow_shared(&mut self, name: &str, span: (usize, usize)) {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(state) = scope.bindings.get_mut(name) {
                match *state {
                    OwnershipState::Moved => {
                        self.diagnostics.push(SemanticDiagnostic {
                            kind: SemanticErrorKind::UseAfterMove,
                            symbol_name: name.to_string(),
                            span,
                            message: format!("Cannot borrow `{}` after move.", name),
                            suggested_fix: Some(format!("Clone or preserve original reference for `{}`", name)),
                        });
                    }
                    OwnershipState::MutablyBorrowed => {
                        self.diagnostics.push(SemanticDiagnostic {
                            kind: SemanticErrorKind::MutableBorrowWithSharedBorrow,
                            symbol_name: name.to_string(),
                            span,
                            message: format!("Cannot borrow `{}` as shared while mutably borrowed.", name),
                            suggested_fix: None,
                        });
                    }
                    OwnershipState::SharedBorrowed(ref mut count) => {
                        *count += 1;
                    }
                    OwnershipState::Owned => {
                        *state = OwnershipState::SharedBorrowed(1);
                    }
                    OwnershipState::Uninitialized => {
                        self.diagnostics.push(SemanticDiagnostic {
                            kind: SemanticErrorKind::UninitializedVariableRead,
                            symbol_name: name.to_string(),
                            span,
                            message: format!("Cannot borrow uninitialized variable `{}`.", name),
                            suggested_fix: None,
                        });
                    }
                }
                return;
            }
        }
    }

    pub fn record_borrow_mut(&mut self, name: &str, span: (usize, usize)) {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(state) = scope.bindings.get_mut(name) {
                match *state {
                    OwnershipState::SharedBorrowed(_) => {
                        self.diagnostics.push(SemanticDiagnostic {
                            kind: SemanticErrorKind::MultipleMutableBorrows,
                            symbol_name: name.to_string(),
                            span,
                            message: format!("Cannot borrow `{}` as mutable when shared borrow exists.", name),
                            suggested_fix: None,
                        });
                    }
                    OwnershipState::MutablyBorrowed => {
                        self.diagnostics.push(SemanticDiagnostic {
                            kind: SemanticErrorKind::MultipleMutableBorrows,
                            symbol_name: name.to_string(),
                            span,
                            message: format!("Cannot borrow `{}` as mutable more than once at a time.", name),
                            suggested_fix: None,
                        });
                    }
                    OwnershipState::Moved => {
                        self.diagnostics.push(SemanticDiagnostic {
                            kind: SemanticErrorKind::UseAfterMove,
                            symbol_name: name.to_string(),
                            span,
                            message: format!("Cannot mutably borrow `{}` after move.", name),
                            suggested_fix: None,
                        });
                    }
                    OwnershipState::Owned => {
                        *state = OwnershipState::MutablyBorrowed;
                    }
                    OwnershipState::Uninitialized => {
                        self.diagnostics.push(SemanticDiagnostic {
                            kind: SemanticErrorKind::UninitializedVariableRead,
                            symbol_name: name.to_string(),
                            span,
                            message: format!("Cannot mutably borrow uninitialized variable `{}`.", name),
                            suggested_fix: None,
                        });
                    }
                }
                return;
            }
        }
    }

    pub fn release_borrow(&mut self, name: &str) {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(state) = scope.bindings.get_mut(name) {
                match *state {
                    OwnershipState::MutablyBorrowed => {
                        *state = OwnershipState::Owned;
                    }
                    OwnershipState::SharedBorrowed(count) => {
                        if count <= 1 {
                            *state = OwnershipState::Owned;
                        } else {
                            *state = OwnershipState::SharedBorrowed(count - 1);
                        }
                    }
                    _ => {}
                }
                return;
            }
        }
    }

    pub fn diagnostics(&self) -> &[SemanticDiagnostic] {
        &self.diagnostics
    }

    pub fn has_errors(&self) -> bool {
        !self.diagnostics.is_empty()
    }
}
