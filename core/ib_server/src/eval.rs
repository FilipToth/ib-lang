use ibc::{analysis::binding::bound_node::BoundNode, eval::{evaluator, IBEval}};

struct ServerEvaluator;

impl IBEval for ServerEvaluator {
    async fn eval(root: &BoundNode) {
        evaluator::eval(root, &mut ServerEvaluator);
    }

    async fn input() -> String {
        "".to_string();
    }

    async fn output(msg: String) {
        
    }
}