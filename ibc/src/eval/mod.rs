use crate::analysis::binding::bound_node::BoundNode;

pub mod evaluator;

pub fn eval(root: &BoundNode) {
    evaluator::eval(root);
}
