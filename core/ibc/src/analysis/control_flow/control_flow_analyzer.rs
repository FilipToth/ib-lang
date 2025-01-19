use std::{cell::RefCell, rc::Rc};

use crate::analysis::{
    binding::types::TypeKind,
    error_bag::{ErrorBag, ErrorKind},
    span::Span,
};

use super::control_flow_graph::ControlFlowNode;

fn analyze_func_rec(
    node: Rc<RefCell<ControlFlowNode>>,
    span: &Span,
    func_ret_type: &TypeKind,
    errors: &mut ErrorBag,
) {
    // walk the tree, all branches should connect to an end node
    // there should be no node where .next is None
    let node = node.borrow();
    if node.is_end {
        return;
    }

    if let Some(on_condition) = &node.on_condition {
        // we don't have to check for next return
        // since the next token is always a block
        let next_ref = on_condition.clone();
        analyze_func_rec(next_ref, span, func_ret_type, errors);
    }

    if let Some(next) = &node.next {
        let next_node = next.borrow();
        let next_ref = next.clone();

        if next_node.is_end {
            // check ret type
            let Some(ret_type) = &node.ret_type else {
                unreachable!()
            };

            // we can return since next node is end
            if ret_type == func_ret_type {
                return;
            }

            let kind = ErrorKind::ReturnTypeMismatch {
                found: ret_type.clone(),
                expected: func_ret_type.clone(),
            };

            errors.add(kind, span.clone());
            return;
        }

        analyze_func_rec(next_ref, span, func_ret_type, errors);
    } else {
        let void = TypeKind::Void;
        if func_ret_type == &void {
            // skip if it's a void function
            return;
        }

        // report error
        let kind = ErrorKind::NotAllCodePathsReturn;
        errors.add(kind, span.clone());
    }
}

pub fn analyze_func(
    root: Rc<RefCell<ControlFlowNode>>,
    loc: &Span,
    func_ret_type: &TypeKind,
    errors: &mut ErrorBag,
) {
    // check if all paths return a value
    analyze_func_rec(root, loc, func_ret_type, errors);
}
