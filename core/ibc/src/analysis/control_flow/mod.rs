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
    label: String,
}

/// One drawable graph: a function's, or the program's own.
pub struct ControlFlowGraph {
    /// What the graph is of, shown on the cluster that holds it.
    pub label: String,
    pub root: Rc<RefCell<ControlFlowNode>>,
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
                label: node.to_string(),
            };

            functions.push(func);
        }
        _ => {}
    }
}

/// The graphs as one Graphviz digraph, each in a cluster of its own so the
/// functions are drawn as separate boxes rather than one run-on graph.
pub fn dot(graphs: &Vec<ControlFlowGraph>) -> String {
    let mut dot_graph = "digraph controlflow {\n".to_string();

    for (index, graph) in graphs.iter().enumerate() {
        let label = graph.label.replace('"', "\\\"");

        dot_graph += format!("  subgraph cluster_{} {{\n", index).as_str();
        dot_graph += format!("    label=\"{}\"\n", label).as_str();
        dot_graph += graph.root.borrow().dot_graph(false).as_str();
        dot_graph += "  }\n";
    }

    dot_graph += "}";
    dot_graph
}

pub fn digraph(graphs: &Vec<ControlFlowGraph>, path: &str) {
    fs::write(path, dot(graphs)).expect("Cannot write to file");
}

pub fn analyze(root: &BoundNode, errors: &mut ErrorBag) -> Vec<ControlFlowGraph> {
    let mut function_declarations: Vec<FuncControlFlow> = Vec::new();
    scan_for_functions_recursive(root, errors, &mut function_declarations);

    // the statements outside any function are a flow of their own. they are
    // only drawn, not analyzed: "not all code paths return" is a question about
    // a function, and the program has no return type to check against
    let program = ControlFlowGraph {
        label: "<program>".to_string(),
        root: control_flow_graph::contruct_graph(root, "p".to_string()),
    };

    let mut graphs: Vec<ControlFlowGraph> = vec![program];

    for (index, func) in function_declarations.into_iter().enumerate() {
        let span = func.span.clone();
        let ret_type = func.ret_type.clone();
        let label = func.label.clone();

        let root = control_flow_graph::contruct_graph(&func.block, format!("f{}", index));
        control_flow_analyzer::analyze_func(root.clone(), &span, &ret_type, errors);

        graphs.push(ControlFlowGraph {
            label: label,
            root: root,
        });
    }

    graphs
}
