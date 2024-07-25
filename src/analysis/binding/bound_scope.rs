use std::{cell::RefCell, rc::Rc};

use super::{binder::BoundParameter, types::TypeKind};

#[derive(Debug)]
pub struct BoundScope {
    parent: Option<Rc<RefCell<BoundScope>>>,
    variables: Vec<VariableDefinition>,
    functions: Vec<FunctionDefinition>,
}

impl BoundScope {
    pub fn new(parent: Rc<RefCell<BoundScope>>) -> BoundScope {
        BoundScope {
            parent: Some(parent),
            variables: Vec::new(),
            functions: Vec::new(),
        }
    }

    pub fn new_root() -> BoundScope {
        BoundScope {
            parent: None,
            variables: Vec::new(),
            functions: Vec::new(),
        }
    }

    pub fn assign_variable(&mut self, identifier: String, var_type: TypeKind) -> bool {
        let existing = self.get_variable(identifier.clone());
        match existing {
            Some(def) => def.var_type == var_type,
            None => {
                let def = VariableDefinition {
                    identifier: identifier,
                    var_type: var_type,
                };

                self.variables.push(def);
                true
            }
        }
    }

    pub fn get_variable(&self, identifier: String) -> Option<VariableDefinition> {
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

    pub fn declare_function(&mut self, identifier: String, params: Vec<BoundParameter>) -> bool {
        let existing = self.get_function(identifier.clone());
        match existing {
            Some(_) => return false,
            None => {
                let def = FunctionDefinition {
                    identifier: identifier.clone(),
                    parameters: params,
                };

                self.functions.push(def);
                true
            }
        }
    }

    pub fn get_function(&self, identifier: String) -> Option<FunctionDefinition> {
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

#[derive(Debug, Clone)]
pub struct VariableDefinition {
    pub identifier: String,
    pub var_type: TypeKind,
}

#[derive(Debug, Clone)]
pub struct FunctionDefinition {
    pub identifier: String,
    pub parameters: Vec<BoundParameter>,
}
