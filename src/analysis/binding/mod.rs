use super::{error_bag::ErrorBag, parser::SyntaxToken, CodeLocation};

pub mod binder;
pub mod types;

pub fn bind_root(root: &SyntaxToken, errors: &mut ErrorBag) -> Option<binder::BoundNode> {
    let root_loc = CodeLocation::new(0, 0);
    binder::bind(root, errors, &root_loc)
}
