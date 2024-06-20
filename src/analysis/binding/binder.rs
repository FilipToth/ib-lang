use crate::analysis::{
    error_bag::ErrorBag,
    operator::Operator,
    parser::{SyntaxKind, SyntaxToken},
    CodeLocation,
};

use super::types::TypeKind;

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
pub struct BoundScope {
    parent: Box<BoundScope>,
    variables: Vec<VariableDefinition>,
}

impl BoundScope {
    pub fn new(parent: BoundScope) -> BoundScope {
        BoundScope {
            parent: Box::new(parent),
            variables: Vec::new(),
        }
    }

    pub fn declare_variable(
        &mut self,
        identifier: String,
        var_type: TypeKind,
        errors: &mut ErrorBag,
    ) -> Option<VariableDefinition> {
        let mut matching = self.variables.to_vec();
        matching.retain(|v| v.identifier == identifier);

        if matching.len() != 0 {
            return None;
        }

        let def = VariableDefinition {
            identifier: identifier,
            var_type: var_type,
        };
        Some(def)
    }
}

#[derive(Debug, Clone)]
pub struct VariableDefinition {
    identifier: String,
    var_type: TypeKind,
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

fn bind_module(
    children: &Vec<SyntaxToken>,
    errors: &mut ErrorBag,
    loc: CodeLocation,
) -> Option<BoundNode> {
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

    let node = BoundNode::new(kind, TypeKind::Void, loc);
    Some(node)
}

fn bind_binary_expression(
    lhs: &SyntaxToken,
    op: &Operator,
    rhs: &SyntaxToken,
    errors: &mut ErrorBag,
    loc: CodeLocation,
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

    let node = BoundNode::new(kind, op_type, loc);
    Some(node)
}

fn bind_unary_expression(
    op: &Operator,
    rhs: &SyntaxToken,
    errors: &mut ErrorBag,
    loc: CodeLocation,
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

pub fn bind(token: &SyntaxToken, errors: &mut ErrorBag) -> Option<BoundNode> {
    let loc = token.loc.clone();
    match &token.kind {
        SyntaxKind::Module { children } => bind_module(&children, errors, loc),
        SyntaxKind::BinaryExpression { lhs, op, rhs } => {
            bind_binary_expression(&lhs, &op, &rhs, errors, loc)
        }
        SyntaxKind::UnaryExpression { op, rhs } => bind_unary_expression(&op, &rhs, errors, loc),
        SyntaxKind::LiteralExpression(subtoken) => bind_literal_expression(&subtoken, errors, loc),
        _ => unreachable!(),
    }
}
