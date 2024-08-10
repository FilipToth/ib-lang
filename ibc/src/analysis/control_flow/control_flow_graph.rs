use std::{cell::RefCell, rc::Rc};

use crate::analysis::binding::{
    bound_node::{BoundNode, BoundNodeKind},
    types::TypeKind,
};

use super::FuncControlFlow;

pub struct ControlFlowNode {
    pub is_start: bool,
    pub is_end: bool,
    pub next: Option<Rc<RefCell<ControlFlowNode>>>,
    pub on_condition: Option<Rc<RefCell<ControlFlowNode>>>,
    pub ret_type: Option<TypeKind>,
    graph_label: String,
    graph_id: String,
}

pub struct ControlFlowSpan {
    first: Rc<RefCell<ControlFlowNode>>,
    last: Rc<RefCell<ControlFlowNode>>,
}

impl ControlFlowSpan {
    pub fn new(
        first: Rc<RefCell<ControlFlowNode>>,
        last: Rc<RefCell<ControlFlowNode>>,
    ) -> ControlFlowSpan {
        ControlFlowSpan {
            first: first,
            last: last,
        }
    }
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
            ret_type: None,
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

            let conn = (node_id.clone(), next_id);
            if !connections.contains(&conn) {
                connections.push(conn);
            }
        }

        if let Some(on_cond) = &self.on_condition {
            let on_cond_id =
                on_cond
                    .borrow_mut()
                    .dot_recursive(nodes, connections, conns_condition);

            let conn = (node_id.clone(), on_cond_id);
            if !conns_condition.contains(&conn) {
                conns_condition.push(conn);
            }
        }

        return node_id;
    }
}

fn walk(
    node: &BoundNode,
    prev: Option<Rc<RefCell<ControlFlowNode>>>,
    end_node: Rc<RefCell<ControlFlowNode>>,
    counter: Rc<RefCell<u64>>,
) -> ControlFlowSpan {
    let (next, last) = match &node.kind {
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

                let last = child_node.last.clone();
                new_prev = last;
            }

            // new prev is last
            (block_ptr.clone(), new_prev)
        }
        BoundNodeKind::ReturnStatement { expr: _ } => {
            let mut new_node = ControlFlowNode::new(counter, node.to_string());

            new_node.next = Some(end_node);
            new_node.ret_type = Some(node.node_type.clone());

            let node_ref = Rc::new(RefCell::new(new_node));
            (node_ref.clone(), node_ref)
        }
        BoundNodeKind::IfStatement {
            condition: _,
            block,
            else_block,
        } => {
            let mut if_node = ControlFlowNode::new(counter.clone(), node.to_string());
            let end_if_node = ControlFlowNode::new(counter.clone(), "end if".to_string());
            let end_if_ref = Rc::new(RefCell::new(end_if_node));

            let on_condition_span = walk(&block, None, end_node.clone(), counter.clone());
            if_node.on_condition = Some(on_condition_span.first);

            let mut on_cond_last = on_condition_span.last.borrow_mut();
            if let None = on_cond_last.next {
                // this path doesn't explicitly return,
                // thus we connect it to the end if
                on_cond_last.next = Some(end_if_ref.clone());
            }

            match else_block {
                Some(e) => {
                    let span = walk(&e, None, end_node.clone(), counter.clone());
                    if_node.next = Some(span.first);

                    let mut last = span.last.borrow_mut();
                    if let None = last.next {
                        // again, this should be connected to end if
                        last.next = Some(end_if_ref.clone());
                    }
                }
                None => if_node.next = Some(end_if_ref.clone()),
            };

            let if_ref = Rc::new(RefCell::new(if_node));
            (if_ref, end_if_ref)
        }
        _ => {
            let node = ControlFlowNode::new(counter, node.to_string());
            let node_ref = Rc::new(RefCell::new(node));
            (node_ref.clone(), node_ref)
        }
    };

    if let Some(prev_ptr) = prev {
        prev_ptr.borrow_mut().next = Some(next.clone());
    }

    ControlFlowSpan::new(next, last)
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
