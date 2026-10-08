//! AST Visitor pattern implementation for recursive node traversal.

use crate::parser::AstDeclaration;

pub trait AstVisitor {
    fn visit_declaration(&mut self, decl: &AstDeclaration);
}

pub struct FunctionCollector {
    pub function_names: Vec<String>,
}

impl FunctionCollector {
    pub fn new() -> Self {
        Self {
            function_names: Vec::new(),
        }
    }
}

impl AstVisitor for FunctionCollector {
    fn visit_declaration(&mut self, decl: &AstDeclaration) {
        if decl.kind == "fn" {
            self.function_names.push(decl.name.clone());
        }
    }
}
