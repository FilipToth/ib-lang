use std::{cell::RefCell, rc::Rc};

use super::{
    bound_node::BoundParameter,
    symbols::{FunctionSymbol, GlobalSymbolScope, VariableSymbol},
    types::TypeKind,
};

#[derive(Debug)]
pub struct BoundScope {
    parent: Option<Rc<RefCell<BoundScope>>>,
    symbol_scope: Rc<RefCell<GlobalSymbolScope>>,
    variables: Vec<VariableSymbol>,
    functions: Vec<FunctionSymbol>,
}

impl BoundScope {
    pub fn new(parent: Rc<RefCell<BoundScope>>) -> BoundScope {
        let sym_scope = parent.borrow().symbol_scope.clone();

        BoundScope {
            parent: Some(parent),
            symbol_scope: sym_scope,
            variables: Vec::new(),
            functions: Vec::new(),
        }
    }

    pub fn new_root() -> BoundScope {
        let sym_scope = GlobalSymbolScope::new();
        let sym_scope_ref = Rc::new(RefCell::new(sym_scope));

        BoundScope {
            parent: None,
            symbol_scope: sym_scope_ref,
            variables: Vec::new(),
            functions: Vec::new(),
        }
    }

    pub fn assign_variable(&mut self, identifier: String, var_type: TypeKind) -> Option<VariableSymbol> {
        let existing = self.get_variable(identifier.clone());
        match existing {
            Some(symbol) => {
                if symbol.var_type == var_type {
                    Some(symbol)
                } else {
                    None
                }
            },
            None => {
                let mut sym_scope = self.symbol_scope.borrow_mut();
                let symbol = sym_scope.alloc_variable(identifier, var_type);

                self.variables.push(symbol.clone());
                Some(symbol)
            }
        }
    }

    pub fn get_variable(&self, identifier: String) -> Option<VariableSymbol> {
        let mut matching = self.variables.to_vec();
        matching.retain(|v| v.identifier == identifier);

        if matching.len() == 0 {
            match &self.parent {
                Some(parent) => parent.borrow().get_variable(identifier),
                None => None,
            }
        } else {
            let def = matching.first().unwrap();
            Some(def.clone())
        }
    }

    pub fn declare_function(
        &mut self,
        identifier: String,
        params: Vec<BoundParameter>,
        ret_type: TypeKind,
    ) -> Option<FunctionSymbol> {
        let existing = self.get_function(identifier.clone());
        match existing {
            Some(_) => return None,
            None => {
                let mut sym_scope = self.symbol_scope.borrow_mut();
                let symbol = sym_scope.alloc_function(identifier, params, ret_type);

                self.functions.push(symbol.clone());
                Some(symbol)
            }
        }
    }

    pub fn get_function(&self, identifier: String) -> Option<FunctionSymbol> {
        let mut matching = self.functions.to_vec();
        matching.retain(|f| f.identifier == identifier);

        if matching.len() == 0 {
            match &self.parent {
                Some(parent) => parent.borrow().get_function(identifier),
                None => None,
            }
        } else {
            let def = matching.first().unwrap();
            Some(def.clone())
        }
    }
}
