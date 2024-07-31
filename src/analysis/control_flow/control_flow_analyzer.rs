use std::{cell::RefCell, rc::Rc};

use crate::analysis::{
    binding::types::TypeKind,
    error_bag::{ErrorBag, ErrorKind},
    CodeLocation,
};

use super::control_flow_graph::ControlFlowNode;

fn analyze_func_rec(
    node: Rc<RefCell<ControlFlowNode>>,
    loc: &CodeLocation,
    func_ret_type: &TypeKind,
    errors: &mut ErrorBag,
) {
    // walk the tree, all branches should connect to an end node
    // there should be no node where .next is None
    let node = node.borrow();
    if node.is_end {
        return;
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

            errors.add(kind, loc.line, loc.col);
            return;
        }

        analyze_func_rec(next_ref, loc, func_ret_type, errors);
    } else {
        // report error
        let kind = ErrorKind::NotAllCodePathsReturn;
        errors.add(kind, loc.line, loc.col);
    }
}

pub fn analyze_func(
    root: Rc<RefCell<ControlFlowNode>>,
    loc: &CodeLocation,
    func_ret_type: &TypeKind,
    errors: &mut ErrorBag,
) {
    // check if all paths return a value
    analyze_func_rec(root, loc, func_ret_type, errors);
}
