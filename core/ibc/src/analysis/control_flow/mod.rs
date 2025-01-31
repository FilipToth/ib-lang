use std::{cell::RefCell, fs, rc::Rc, sync::Arc};

use self::control_flow_graph::ControlFlowNode;

use super::{
    binding::{
        bound_node::{BoundNode, BoundNodeKind},
        types::TypeKind,
    },
    error_bag::ErrorBag,
    span::Span,
};

pub mod control_flow_analyzer;
pub mod control_flow_graph;

pub struct FuncControlFlow {
    block: Arc<BoundNode>,
    ret_type: TypeKind,
    span: Span,
}

fn scan_for_functions_recursive(
    node: &BoundNode,
    errors: &mut ErrorBag,
    functions: &mut Vec<FuncControlFlow>,
) {
    match &node.kind {
        BoundNodeKind::Module { block } => {
            scan_for_functions_recursive(&block, errors, functions);
        }
        BoundNodeKind::Block { children } => {
            for child in children.iter() {
                scan_for_functions_recursive(&child, errors, functions);
            }
        }
        BoundNodeKind::IfStatement {
            condition: _,
            block,
            else_block,
        } => {
            scan_for_functions_recursive(block, errors, functions);
            if let Some(e) = else_block {
                scan_for_functions_recursive(e, errors, functions)
            }
        }
        BoundNodeKind::FunctionDeclaration { symbol, block } => {
            let func = FuncControlFlow {
                block: block.clone(),
                ret_type: symbol.ret_type.clone(),
                span: node.span.clone(),
            };

            functions.push(func);
        }
        _ => {}
    }
}

pub fn digraph(graphs: &Vec<Rc<RefCell<ControlFlowNode>>>, path: &str) {
    let mut dot_graph = "".to_string();
    dot_graph += "digraph controlflow {";

    for graph in graphs {
        let subgraph = graph.borrow().dot_graph(false);
        dot_graph += subgraph.as_str();
    }

    dot_graph += "}";
    fs::write(path, dot_graph).expect("Cannot write to file");
}

pub fn analyze(root: &BoundNode, errors: &mut ErrorBag) -> Vec<Rc<RefCell<ControlFlowNode>>> {
    let mut function_declarations: Vec<FuncControlFlow> = Vec::new();
    scan_for_functions_recursive(root, errors, &mut function_declarations);

    let mut graphs: Vec<Rc<RefCell<ControlFlowNode>>> = Vec::new();
    for func in function_declarations {
        let span = func.span.clone();
        let ret_type = func.ret_type.clone();

        let graph = control_flow_graph::contruct_graph(func);
        control_flow_analyzer::analyze_func(graph.clone(), &span, &ret_type, errors);
        graphs.push(graph);
    }

    graphs
}
