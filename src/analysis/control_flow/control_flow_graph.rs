use std::{cell::RefCell, rc::Rc};

use crate::analysis::binding::bound_node::{BoundNode, BoundNodeKind};

use super::FuncControlFlow;

pub struct ControlFlowNode {
    pub is_start: bool,
    pub is_end: bool,
    pub next: Option<Rc<RefCell<ControlFlowNode>>>,
    pub on_condition: Option<Rc<RefCell<ControlFlowNode>>>,
    graph_label: String,
    graph_id: String,
}

impl ControlFlowNode {
    fn new(counter: Rc<RefCell<u64>>, label: String) -> ControlFlowNode {
        let mut count = counter.borrow_mut();
        *count += 1;

        ControlFlowNode {
            is_start: false,
            is_end: false,
            next: None,
            on_condition: None,
            graph_label: label,
            graph_id: count.to_string(),
        }
    }

    pub fn dot_graph(&self, include_header: bool) -> String {
        let mut nodes: Vec<(String, String)> = Vec::new();
        let mut connections: Vec<(String, String)> = Vec::new();
        let mut conns_condition: Vec<(String, String)> = Vec::new();
        self.dot_recursive(&mut nodes, &mut connections, &mut conns_condition);

        let mut graph = "".to_string();

        if include_header {
            graph += "digraph controlflow {\n";
        }

        for (node_id, label) in nodes {
            graph += format!("    {} [label=\"{}\"]\n", node_id, label).as_str();
        }

        for (from, to) in connections {
            graph += format!("    {} -> {}\n", from, to).as_str();
        }

        for (from, to) in conns_condition {
            graph += format!("    {} -> {} [label=\"<condition>\"]\n", from, to).as_str();
        }

        if include_header {
            graph += "}";
        }

        graph
    }

    fn dot_recursive(
        &self,
        nodes: &mut Vec<(String, String)>,
        connections: &mut Vec<(String, String)>,
        conns_condition: &mut Vec<(String, String)>,
    ) -> String {
        // add node
        let node_id = self.graph_id.clone();
        let node_label = self.graph_label.clone();

        nodes.push((node_id.clone(), node_label));

        if let Some(next) = &self.next {
            let next_id = next
                .borrow_mut()
                .dot_recursive(nodes, connections, conns_condition);

            connections.push((node_id.clone(), next_id));
        }

        if let Some(on_cond) = &self.on_condition {
            let on_cond_id =
                on_cond
                    .borrow_mut()
                    .dot_recursive(nodes, connections, conns_condition);

            conns_condition.push((node_id.clone(), on_cond_id));
        }

        return node_id;
    }
}

fn walk(
    node: &BoundNode,
    prev: Option<Rc<RefCell<ControlFlowNode>>>,
    end_node: Rc<RefCell<ControlFlowNode>>,
    counter: Rc<RefCell<u64>>,
) -> Rc<RefCell<ControlFlowNode>> {
    let node = match &node.kind {
        BoundNodeKind::Block { children } => {
            let block = ControlFlowNode::new(counter.clone(), node.to_string());
            let block_ptr = Rc::new(RefCell::new(block));

            let mut new_prev = block_ptr.clone();
            for child in children.iter() {
                let child_node = walk(
                    child,
                    Some(new_prev.clone()),
                    end_node.clone(),
                    counter.clone(),
                );
                new_prev = child_node.clone();
            }

            block_ptr
        }
        BoundNodeKind::ReturnStatement { expr: _ } => {
            let mut node = ControlFlowNode::new(counter, node.to_string());
            node.next = Some(end_node);

            Rc::new(RefCell::new(node))
        }
        BoundNodeKind::IfStatement {
            condition: _,
            block,
        } => {
            let mut node = ControlFlowNode::new(counter.clone(), node.to_string());

            let on_condition = walk(&block, None, end_node, counter);
            node.on_condition = Some(on_condition.clone());

            Rc::new(RefCell::new(node))
        }
        _ => {
            let node = ControlFlowNode::new(counter, node.to_string());
            Rc::new(RefCell::new(node))
        }
    };

    if let Some(prev_ptr) = prev {
        prev_ptr.borrow_mut().next = Some(node.clone());
    }

    node
}

pub fn contruct_graph(func: FuncControlFlow) -> Rc<RefCell<ControlFlowNode>> {
    let counter = Rc::new(RefCell::new(0 as u64));

    let mut start_node = ControlFlowNode::new(counter.clone(), "<Start>".to_string());
    start_node.is_start = true;
    let start_node_ref = Rc::new(RefCell::new(start_node));

    let mut end_node = ControlFlowNode::new(counter.clone(), "<End>".to_string());
    end_node.is_end = true;
    let end_node_ref = Rc::new(RefCell::new(end_node));

    let start = func.block;
    walk(&start, Some(start_node_ref.clone()), end_node_ref, counter);

    start_node_ref
}
