use crate::analysis::binding::bound_node::BoundNode;
use async_trait::async_trait;

pub mod evaluator;
pub mod object_methods;
pub mod eval_builtin;

#[async_trait]
pub trait IBEval {
    async fn eval(&mut self, root: &BoundNode);
    fn output(&mut self, msg: String);
    async fn input(&mut self) -> String;
}

/* struct Evaluator;

impl Evaluator {
    pub async fn eval(root: &BoundNode) -> String {
        let mut msg = String::from("");
        evaluator::eval(root, self);
    
        msg.trim().to_string()
    }
} */