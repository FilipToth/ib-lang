use pest::iterators::Pair;

use crate::analysis::{operator::Operator, CodeLocation};

use super::parser;

#[derive(Debug)]
pub struct SyntaxToken {
    pub kind: SyntaxKind,
    pub loc: CodeLocation,
}

impl SyntaxToken {
    pub fn new(kind: SyntaxKind, rule: &Pair<parser::Rule>) -> SyntaxToken {
        let location = CodeLocation::from_pair(rule);
        SyntaxToken {
            kind: kind,
            loc: location,
        }
    }
}

#[derive(Debug)]
pub enum SyntaxKind {
    Module {
        block: Box<SyntaxToken>,
    },
    Block {
        children: Box<Vec<SyntaxToken>>,
    },
    IfStatement {
        condition: Box<SyntaxToken>,
        block: Box<SyntaxToken>,
        else_block: Option<Box<SyntaxToken>>,
    },
    OutputStatement {
        expr: Box<SyntaxToken>,
    },
    ReturnStatement {
        expr: Option<Box<SyntaxToken>>,
    },
    FunctionDeclaration {
        identifier: Box<SyntaxToken>,
        params: Box<SyntaxToken>,
        ret_type: String,
        block: Box<SyntaxToken>,
    },
    FunctionTypeAnnotation(String),
    ParameterList {
        params: Vec<ParameterSyntax>,
    },
    AssignmentExpression {
        reference: Box<SyntaxToken>,
        value: Box<SyntaxToken>,
    },
    CallExpression {
        identifier: Box<SyntaxToken>,
        args: Box<Vec<SyntaxToken>>,
    },
    ReferenceExpression(String),
    BinaryExpression {
        lhs: Box<SyntaxToken>,
        op: Operator,
        rhs: Box<SyntaxToken>,
    },
    UnaryExpression {
        op: Operator,
        rhs: Box<SyntaxToken>,
    },
    LiteralExpression(Box<SyntaxToken>),
    IdentifierToken(String),
    NumberToken(i32),
    BooleanToken(bool),
}

#[derive(Debug)]
pub struct ParameterSyntax {
    pub identifier: String,
    pub type_annotation: String,
    pub location: CodeLocation,
}
