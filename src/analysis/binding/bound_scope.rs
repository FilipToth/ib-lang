use std::{cell::RefCell, rc::Rc};

use super::types::TypeKind;

#[derive(Debug)]
pub struct BoundScope {
    parent: Option<Rc<RefCell<BoundScope>>>,
    variables: Vec<VariableDefinition>,
}

impl BoundScope {
    pub fn new(parent: BoundScope) -> BoundScope {
        BoundScope {
            parent: Some(Rc::new(RefCell::new(parent))),
            variables: Vec::new(),
        }
    }

    pub fn new_root() -> BoundScope {
        BoundScope {
            parent: None,
            variables: Vec::new(),
        }
    }

    pub fn assign(&mut self, identifier: String, var_type: TypeKind) -> bool {
        let existing = self.get(identifier.clone());
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

    pub fn get(&self, identifier: String) -> Option<VariableDefinition> {
        let mut matching = self.variables.to_vec();
        matching.retain(|v| v.identifier == identifier);

        if matching.len() == 0 {
            match &self.parent {
                Some(parent) => parent.borrow().get(identifier),
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
