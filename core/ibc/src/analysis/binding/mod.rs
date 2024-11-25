use std::{cell::RefCell, rc::Rc};

use super::{error_bag::ErrorBag, syntax::syntax_token::SyntaxToken};

pub mod binder;
pub mod bound_node;
mod bound_scope;
pub mod symbols;
pub mod types;

pub fn bind_root(root: &SyntaxToken, errors: &mut ErrorBag) -> Option<bound_node::BoundNode> {
    // yes there will be two root scopes, but this is
    // just a minor inefficiency

    let scope = bound_scope::BoundScope::new_root();
    binder::bind(root, Rc::new(RefCell::new(scope)), errors)
}
