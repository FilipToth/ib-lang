use crate::analysis::{operator::Operator, span::Span};

#[derive(Debug)]
pub struct SyntaxToken {
    pub kind: SyntaxKind,
    pub span: Span,
}

impl SyntaxToken {
    pub fn new(kind: SyntaxKind, span: Span) -> SyntaxToken {
        SyntaxToken {
            kind: kind,
            span: span,
        }
    }
}

#[derive(Debug)]
pub enum SyntaxKind {
    Scope {
        subtokens: Vec<SyntaxToken>,
    },
    ReferenceExpression(String),
    ObjectMemberExpression {
        base: Box<SyntaxToken>,
        next: Box<SyntaxToken>,
    },
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
    InstantiationExpression {
        type_name: String,
        type_param: Option<String>,
        args: Vec<SyntaxToken>,
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
    ForLoop {
        identifier: String,
        lower_bound: usize,
        upper_bound: usize,
        body: Box<SyntaxToken>,
    },
}
