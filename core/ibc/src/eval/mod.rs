use crate::analysis::binding::bound_node::BoundNode;
use async_trait::async_trait;

pub mod evaluator;
pub mod object_methods;
pub mod eval_builtin;

#[async_trait]
pub trait IBEval: Send + Sync {
    async fn eval(root: &BoundNode);
    fn output(msg: String);
    async fn input() -> String;
}

/* struct Evaluator;

impl Evaluator {
    pub async fn eval(root: &BoundNode) -> String {
        let mut msg = String::from("");
        evaluator::eval(root, self);
    
        msg.trim().to_string()
    }
} */