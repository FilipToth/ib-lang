use crate::analysis::binding::bound_node::BoundNode;
use async_trait::async_trait;

pub mod eval_builtin;
pub mod evaluator;
pub mod object_methods;

#[async_trait]
pub trait EvalIO: Send + Sync {
    async fn output(&self, msg: String);
    async fn input(&self) -> String;
}
