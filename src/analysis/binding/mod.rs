use std::{cell::RefCell, rc::Rc};

use super::{error_bag::ErrorBag, parser::SyntaxToken};

pub mod binder;
mod bound_scope;
pub mod types;

pub fn bind_root(root: &SyntaxToken, errors: &mut ErrorBag) -> Option<binder::BoundNode> {
    let scope = bound_scope::BoundScope::new_root();
    binder::bind(root, Rc::new(RefCell::new(scope)), errors)
}
