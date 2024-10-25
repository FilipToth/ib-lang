use crate::analysis::binding::bound_node::BoundNode;

pub mod evaluator;

pub fn eval(root: &BoundNode) -> String {
    let mut output = String::from("");
    evaluator::eval(root, &mut output);

    output
        .trim()
        .to_string()
}
