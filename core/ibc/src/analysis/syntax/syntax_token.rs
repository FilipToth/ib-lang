use crate::analysis::{operator::Operator, span::Span};

/// A type as written in source, before the binder resolves it: `Int`,
/// `Stack<Int>`, or nested, `Array<Stack<Int>>`.
#[derive(Debug, Clone)]
pub struct TypeAnnotation {
    pub name: String,
    pub generic: Option<Box<TypeAnnotation>>,
    pub span: Span,
}

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
    IndexExpression {
        base: Box<SyntaxToken>,
        index: Box<SyntaxToken>,
    },
    IndexAssignmentExpression {
        base: Box<SyntaxToken>,
        index: Box<SyntaxToken>,
        value: Box<SyntaxToken>,
    },
    IntegerLiteralExpression(i64),
    BooleanLiteralExpression(bool),
    StringLiteralExpression(String),
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
        type_annotation: TypeAnnotation,
        args: Vec<SyntaxToken>,
    },
    OutputStatement {
        exprs: Vec<SyntaxToken>,
    },
    IfStatement {
        condition: Box<SyntaxToken>,
        body: Box<SyntaxToken>,
        else_body: Option<Box<SyntaxToken>>,
    },
    Parameter {
        identifier: String,
        type_annotation: Option<TypeAnnotation>,
    },
    FunctionDeclaration {
        identifier: String,
        parameters: Vec<SyntaxToken>,
        return_type: Option<TypeAnnotation>,
        body: Box<SyntaxToken>,
    },
    ReturnStatement {
        expr: Option<Box<SyntaxToken>>,
    },
    ForLoop {
        identifier: String,
        lower_bound: Box<SyntaxToken>,
        upper_bound: Box<SyntaxToken>,
        body: Box<SyntaxToken>,
    },
    WhileLoop {
        expr: Box<SyntaxToken>,
        body: Box<SyntaxToken>,
    },
    UntilLoop {
        expr: Box<SyntaxToken>,
        body: Box<SyntaxToken>,
    },
}
