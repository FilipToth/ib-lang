use crate::analysis::{operator::Operator, CodeLocation};

#[derive(Debug)]
pub struct SyntaxToken {
    pub kind: SyntaxKind,
    pub loc: CodeLocation,
}

impl SyntaxToken {
    pub fn new(kind: SyntaxKind, loc: CodeLocation) -> SyntaxToken {
        SyntaxToken {
            kind: kind,
            loc: loc,
        }
    }
}

#[derive(Debug)]
pub enum SyntaxKind {
    Scope {
        subtokens: Vec<SyntaxToken>,
    },
    ReferenceExpression(String),
    IntegerLiteralExpression(i64),
    BooleanLiteralExpression(bool),
    BinaryExpression {
        lhs: Box<SyntaxToken>,
        op: Operator,
        rhs: Box<SyntaxToken>,
    },
    UnaryExpression {
        op: Operator,
        rhs: Box<SyntaxToken>,
    },
    CallExpression {
        identifier: String,
        args: Vec<SyntaxToken>,
    },
    AssignmentExpression {
        identifier: String,
        value: Box<SyntaxToken>,
    },
    ParenthesizedExpression {
        inner: Box<SyntaxToken>,
    },
    OutputStatement {
        expr: Box<SyntaxToken>,
    },
    IfStatement {
        condition: Box<SyntaxToken>,
        body: Box<SyntaxToken>,
    },
    Parameter {
        identifier: String,
        type_annotation: String,
    },
    FunctionDeclaration {
        identifier: String,
        parameters: Vec<SyntaxToken>,
        return_type: Option<String>,
        body: Box<SyntaxToken>,
    },
    ReturnStatement {
        expr: Option<Box<SyntaxToken>>,
    },
}
