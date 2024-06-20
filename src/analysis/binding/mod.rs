use super::{error_bag::ErrorBag, parser::SyntaxToken};

pub mod binder;
pub mod types;

pub fn bind_root(root: &SyntaxToken, errors: &mut ErrorBag) -> Option<binder::BoundNode> {
    binder::bind(root, errors)
}
