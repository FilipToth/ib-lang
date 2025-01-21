use crate::analysis::binding::bound_node::BoundNode;

pub mod evaluator;
pub mod object_methods;
pub mod eval_builtin;

pub fn eval(root: &BoundNode, input_handler: fn() -> String) -> String {
    let mut output = String::from("");
    evaluator::eval(root, &mut output, input_handler);

    output.trim().to_string()
}
