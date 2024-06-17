use crate::analysis::{error_bag::ErrorBag, operator::Operator, parser::SyntaxToken};

use super::types::TypeKind;

#[derive(Debug)]
pub struct BoundNode {
    pub kind: BoundNodeKind,
    pub node_type: TypeKind,
}

impl BoundNode {
    pub fn new(kind: BoundNodeKind, node_type: TypeKind) -> BoundNode {
        BoundNode {
            kind: kind,
            node_type: node_type,
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
    NumberLiteral(i32),
    BooleanLiteral(bool),
}

fn bind_module(children: &Vec<SyntaxToken>, errors: &mut ErrorBag) -> Option<BoundNode> {
    let mut bound = Vec::<BoundNode>::new();
    for child in children {
        let bound_child = match bind(child, errors) {
            Some(n) => n,
            None => return None,
        };

        bound.push(bound_child);
    }

    let kind = BoundNodeKind::Module {
        children: Box::new(bound),
    };

    let node = BoundNode::new(kind, TypeKind::Void);
    Some(node)
}

fn bind_binary_expression(
    lhs: &SyntaxToken,
    op: &Operator,
    rhs: &SyntaxToken,
    errors: &mut ErrorBag,
) -> Option<BoundNode> {
    let lhs = match bind(lhs, errors) {
        Some(n) => n,
        None => return None,
    };

    let rhs = match bind(rhs, errors) {
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

    let node = BoundNode::new(kind, op_type);
    Some(node)
}

fn bind_unary_expression(
    op: &Operator,
    rhs: &SyntaxToken,
    errors: &mut ErrorBag,
) -> Option<BoundNode> {
    let rhs = match bind(rhs, errors) {
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

    let node = BoundNode::new(kind, op_type);
    Some(node)
}

fn bind_literal_expression(subtoken: &SyntaxToken, _errors: &mut ErrorBag) -> Option<BoundNode> {
    match subtoken {
        SyntaxToken::NumberToken(number) => {
            let kind = BoundNodeKind::NumberLiteral(number.clone());
            let node = BoundNode::new(kind, TypeKind::Int);
            Some(node)
        }
        SyntaxToken::BooleanToken(value) => {
            let kind = BoundNodeKind::BooleanLiteral(value.clone());
            let node = BoundNode::new(kind, TypeKind::Boolean);
            Some(node)
        }
        _ => unreachable!(),
    }
}

pub fn bind(token: &SyntaxToken, errors: &mut ErrorBag) -> Option<BoundNode> {
    match token {
        SyntaxToken::Module { children } => bind_module(children, errors),
        SyntaxToken::BinaryExpression { lhs, op, rhs } => {
            bind_binary_expression(lhs, op, rhs, errors)
        }
        SyntaxToken::UnaryExpression { op, rhs } => bind_unary_expression(op, rhs, errors),
        SyntaxToken::LiteralExpression(subtoken) => bind_literal_expression(subtoken, errors),
        _ => unreachable!(),
    }
}
