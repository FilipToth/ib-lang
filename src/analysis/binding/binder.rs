use std::{cell::RefCell, rc::Rc};

use crate::analysis::{
    error_bag::{ErrorBag, ErrorKind},
    operator::Operator,
    parser::{SyntaxKind, SyntaxToken},
    CodeLocation,
};

use super::{bound_scope::BoundScope, types::TypeKind};

#[derive(Debug)]
pub struct BoundNode {
    pub kind: BoundNodeKind,
    pub node_type: TypeKind,
    pub loc: CodeLocation,
}

impl BoundNode {
    pub fn new(kind: BoundNodeKind, node_type: TypeKind, loc: CodeLocation) -> BoundNode {
        BoundNode {
            kind: kind,
            node_type: node_type,
            loc: loc,
        }
    }
}

#[derive(Debug)]
pub enum BoundNodeKind {
    Module {
        children: Box<Vec<BoundNode>>,
    },
    BinaryExpression {
        lhs: Box<BoundNode>,
        op: Operator,
        rhs: Box<BoundNode>,
    },
    UnaryExpression {
        op: Operator,
        rhs: Box<BoundNode>,
    },
    AssignmentExpression {
        identifier: String,
        value: Box<BoundNode>,
    },
    NumberLiteral(i32),
    BooleanLiteral(bool),
}

fn bind_module(
    children: &Vec<SyntaxToken>,
    scope: Rc<RefCell<BoundScope>>,
    errors: &mut ErrorBag,
    loc: CodeLocation,
) -> Option<BoundNode> {
    let mut bound = Vec::<BoundNode>::new();
    for child in children {
        let bound_child = match bind(child, scope.clone(), errors) {
            Some(n) => n,
            None => return None,
        };

        bound.push(bound_child);
    }

    let kind = BoundNodeKind::Module {
        children: Box::new(bound),
    };

    let node = BoundNode::new(kind, TypeKind::Void, loc);
    Some(node)
}

fn bind_binary_expression(
    lhs: &SyntaxToken,
    op: &Operator,
    rhs: &SyntaxToken,
    scope: Rc<RefCell<BoundScope>>,
    errors: &mut ErrorBag,
    loc: CodeLocation,
) -> Option<BoundNode> {
    let lhs = match bind(lhs, scope.clone(), errors) {
        Some(n) => n,
        None => return None,
    };

    let rhs = match bind(rhs, scope, errors) {
        Some(n) => n,
        None => return None,
    };

    let op_type = match op.return_type_binary(&lhs, &rhs, errors) {
        Some(t) => t,
        None => return None,
    };

    let kind = BoundNodeKind::BinaryExpression {
        lhs: Box::new(lhs),
        op: op.clone(),
        rhs: Box::new(rhs),
    };

    let node = BoundNode::new(kind, op_type, loc);
    Some(node)
}

fn bind_unary_expression(
    op: &Operator,
    rhs: &SyntaxToken,
    scope: Rc<RefCell<BoundScope>>,
    errors: &mut ErrorBag,
    loc: CodeLocation,
) -> Option<BoundNode> {
    let rhs = match bind(rhs, scope, errors) {
        Some(n) => n,
        None => return None,
    };

    let op_type = match op.return_type_unary(&rhs, errors) {
        Some(t) => t,
        None => return None,
    };

    let kind = BoundNodeKind::UnaryExpression {
        op: op.clone(),
        rhs: Box::new(rhs),
    };

    let node = BoundNode::new(kind, op_type, loc);
    Some(node)
}

fn bind_literal_expression(
    subtoken: &SyntaxToken,
    _errors: &mut ErrorBag,
    loc: CodeLocation,
) -> Option<BoundNode> {
    match subtoken.kind {
        SyntaxKind::NumberToken(number) => {
            let kind = BoundNodeKind::NumberLiteral(number.clone());
            let node = BoundNode::new(kind, TypeKind::Int, loc);
            Some(node)
        }
        SyntaxKind::BooleanToken(value) => {
            let kind = BoundNodeKind::BooleanLiteral(value.clone());
            let node = BoundNode::new(kind, TypeKind::Boolean, loc);
            Some(node)
        }
        _ => unreachable!(),
    }
}

fn bind_assignment_expression(
    reference: &SyntaxToken,
    value: &SyntaxToken,
    scope: Rc<RefCell<BoundScope>>,
    errors: &mut ErrorBag,
    loc: CodeLocation,
) -> Option<BoundNode> {
    let identifier = match &reference.kind {
        SyntaxKind::ReferenceExpression(i) => i.clone(),
        _ => unreachable!(),
    };

    let value = match bind(value, scope.clone(), errors) {
        Some(v) => v,
        None => return None,
    };

    let node_type = value.node_type.clone();
    let success = scope
        .borrow_mut()
        .assign(identifier.clone(), node_type.clone());
    if !success {
        errors.add(ErrorKind::AssignMismatchedTypes, loc.line, loc.col);
        return None;
    }

    let kind = BoundNodeKind::AssignmentExpression {
        identifier: identifier,
        value: Box::new(value),
    };

    let node = BoundNode::new(kind, node_type, loc);
    Some(node)
}

pub fn bind(
    token: &SyntaxToken,
    scope: Rc<RefCell<BoundScope>>,
    errors: &mut ErrorBag,
) -> Option<BoundNode> {
    let loc = token.loc.clone();
    match &token.kind {
        SyntaxKind::Module { children } => bind_module(&children, scope, errors, loc),
        SyntaxKind::BinaryExpression { lhs, op, rhs } => {
            bind_binary_expression(&lhs, &op, &rhs, scope, errors, loc)
        }
        SyntaxKind::UnaryExpression { op, rhs } => {
            bind_unary_expression(&op, &rhs, scope, errors, loc)
        }
        SyntaxKind::LiteralExpression(subtoken) => bind_literal_expression(&subtoken, errors, loc),
        SyntaxKind::AssignmentExpression { reference, value } => {
            bind_assignment_expression(reference, value, scope, errors, loc)
        }
        _ => unreachable!(),
    }
}
