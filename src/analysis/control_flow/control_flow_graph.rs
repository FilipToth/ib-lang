use core::fmt;
use std::{cell::RefCell, fmt::format, rc::Rc};

use crate::analysis::binding::binder::{BoundNode, BoundNodeKind};

use super::FuncControlFlow;

pub struct ControlFlowNode {
    pub is_start: bool,
    pub is_end: bool,
    pub next: Option<Rc<RefCell<ControlFlowNode>>>,
    pub condition: Option<String>,
    pub on_condition: Option<Rc<RefCell<ControlFlowNode>>>,
}

impl ControlFlowNode {
    fn new() -> ControlFlowNode {
        ControlFlowNode {
            is_start: false,
            is_end: false,
            next: None,
            condition: None,
            on_condition: None,
        }
    }

    fn dot_graph(&self) -> String {
        "".to_string()
    }
}

fn walk(node: &BoundNode, prev: Option<Rc<RefCell<ControlFlowNode>>>, end_node: Rc<RefCell<ControlFlowNode>>) -> Rc<RefCell<ControlFlowNode>> {
    let node = match &node.kind {
        BoundNodeKind::Block { children } => {
            let block = ControlFlowNode::new();
            let block_ptr = Rc::new(RefCell::new(block));

            let mut new_prev = block_ptr.clone();
            for child in children.iter() {
                let child_node = walk(child, Some(new_prev.clone()), end_node.clone());
                new_prev = child_node.clone();
            }

            block_ptr
        },
        BoundNodeKind::ReturnStatement { expr: _ } => {
            println!("Return statement...");
            let mut node = ControlFlowNode::new();
            node.next = Some(end_node);

            Rc::new(RefCell::new(node))
        },
        BoundNodeKind::IfStatement { condition: _, block } => {
            println!("If statement");
            let mut node = ControlFlowNode::new();

            let on_condition = walk(&block, None, end_node);
            node.on_condition = Some(on_condition.clone());

            // TODO: fmt condition
            node.condition = Some("Condition".to_string());

            Rc::new(RefCell::new(node))
        }
        _ => {
            println!("Creating ctlflow node");
            let node = ControlFlowNode::new();
            Rc::new(RefCell::new(node))
        },
    };

    if let Some(prev_ptr) = prev {
        prev_ptr.borrow_mut().next = Some(node.clone());
    }

    node
}

pub fn contruct_graph(func: FuncControlFlow) -> Rc<RefCell<ControlFlowNode>> {
    let mut start_node = ControlFlowNode::new();
    start_node.is_start = true;
    let start_node_ref = Rc::new(RefCell::new(start_node));

    let mut end_node = ControlFlowNode::new();
    end_node.is_end = true;
    let end_node_ref = Rc::new(RefCell::new(end_node));

    let start = func.block;
    walk(&start, Some(start_node_ref.clone()), end_node_ref);

    println!("{}", start_node);

    start_node_ref
}
