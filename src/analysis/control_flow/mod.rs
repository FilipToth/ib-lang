use std::rc::Rc;

use super::{
    binding::{
        binder::{BoundNode, BoundNodeKind},
        types::TypeKind,
    },
    error_bag::ErrorBag,
};

pub mod control_flow_graph;

pub struct FuncControlFlow {
    block: Rc<BoundNode>,
    ret_type: TypeKind,
}

fn scan_for_functions_recursive(
    node: &BoundNode,
    errors: &mut ErrorBag,
    functions: &mut Vec<FuncControlFlow>,
) {
    match &node.kind {
        BoundNodeKind::Module { block } => {
            scan_for_functions_recursive(block, errors, functions);
        }
        BoundNodeKind::Block { children } => {
            for child in children.iter() {
                scan_for_functions_recursive(&child, errors, functions);
            }
        }
        BoundNodeKind::IfStatement {
            condition: _,
            block,
        } => {
            scan_for_functions_recursive(block, errors, functions);
        }
        BoundNodeKind::FunctionDeclaration {
            identifier: _,
            params: _,
            ret_type,
            block,
        } => {
            let func = FuncControlFlow {
                block: block.clone(),
                ret_type: ret_type.clone(),
            };

            functions.push(func);
        }
        _ => {}
    }
}

pub fn analyze(root: &BoundNode, errors: &mut ErrorBag) {
    let mut function_declarations: Vec<FuncControlFlow> = Vec::new();
    scan_for_functions_recursive(root, errors, &mut function_declarations);

    for func in function_declarations {
        let graph = control_flow_graph::contruct_graph(func);
    }
}
