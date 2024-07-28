use crate::analysis::binding::binder::BoundNode;

use super::FuncControlFlow;

pub struct ControlFlowNode {
    pub is_start: bool,
    pub is_end: bool,
    pub next: Option<Box<ControlFlowNode>>,
    pub condition: Option<String>,
    pub on_condition: Option<Box<ControlFlowNode>>,
}

pub fn contruct_graph(func: FuncControlFlow) -> ControlFlowNode {

}
