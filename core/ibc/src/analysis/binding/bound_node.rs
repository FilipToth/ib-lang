use std::sync::Arc;

use crate::analysis::{operator::Operator, span::Span};

use super::{
    symbols::{FunctionSymbol, VariableSymbol},
    types::TypeKind,
};

#[derive(Debug)]
pub struct BoundNode {
    pub kind: BoundNodeKind,
    pub node_type: TypeKind,
    pub span: Span,
}

impl BoundNode {
    pub fn new(kind: BoundNodeKind, node_type: TypeKind, span: Span) -> BoundNode {
        BoundNode {
            kind: kind,
            node_type: node_type,
            span: span,
        }
    }

    pub fn to_string(&self) -> String {
        match &self.kind {
            BoundNodeKind::Module { .. } => "Module".to_string(),
            BoundNodeKind::Block { .. } => "Block".to_string(),
            BoundNodeKind::OutputStatement { exprs } => {
                let parts: Vec<String> = exprs.iter().map(|expr| expr.to_string()).collect();
                format!("output {}", parts.join(" , "))
            }
            BoundNodeKind::ReturnStatement { expr } => {
                let expr_fmt = match expr {
                    Some(expr) => format!(" {}", expr.to_string()),
                    None => "".to_string(),
                };

                format!("return{}", expr_fmt)
            }
            BoundNodeKind::IfStatement {
                condition,
                block: _,
                else_block: _,
            } => format!("if {}", &condition.to_string()),
            BoundNodeKind::ForLoop {
                iterator,
                lower_bound,
                upper_bound,
                block,
            } => {
                format!(
                    "loop {} from {} to {} {} end",
                    iterator.name(),
                    lower_bound.to_string(),
                    upper_bound.to_string(),
                    block.to_string()
                )
            }
            BoundNodeKind::WhileLoop { expr, block } => {
                format!("loop while {} {} end", expr.to_string(), block.to_string())
            }
            BoundNodeKind::UntilLoop { expr, block } => {
                format!("loop until {} {} end", expr.to_string(), block.to_string())
            }
            BoundNodeKind::FunctionDeclaration { symbol, block: _ } => {
                format!(
                    "function {}(...) -> {}",
                    symbol.identifier,
                    symbol.ret_type.to_string()
                )
            }
            BoundNodeKind::BinaryExpression { lhs, op, rhs } => {
                format!("{} {} {}", lhs.to_string(), op.to_string(), rhs.to_string())
            }
            BoundNodeKind::UnaryExpression { op, rhs } => {
                format!("{}{}", op.to_string(), rhs.to_string())
            }
            BoundNodeKind::AssignmentExpression { symbol, value } => {
                format!("{} = {}", symbol.name(), value.to_string())
            }
            BoundNodeKind::BoundCallExpression { symbol, args: _ } => {
                format!("{}(...)", symbol.identifier)
            }
            BoundNodeKind::ObjectExpression => "Object".to_string(),
            BoundNodeKind::ObjectMemberExpression { base, next } => {
                format!("{}.{}", base.to_string(), next.to_string())
            }
            BoundNodeKind::IndexExpression { base, index } => {
                format!("{}[{}]", base.to_string(), index.to_string())
            }
            BoundNodeKind::IndexAssignmentExpression { base, index, value } => {
                format!(
                    "{}[{}] = {}",
                    base.to_string(),
                    index.to_string(),
                    value.to_string()
                )
            }
            BoundNodeKind::ReferenceExpression(sym) => sym.name().to_string(),
            BoundNodeKind::NumberLiteral(num) => num.to_string(),
            BoundNodeKind::BooleanLiteral(bool) => bool.to_string(),
            BoundNodeKind::StringLiteral(val) => {
                format!("\\\"{}\\\"", val)
            }
        }
    }
}

#[derive(Debug)]
pub enum BoundNodeKind {
    Module {
        block: Box<BoundNode>,
    },
    Block {
        children: Box<Vec<BoundNode>>,
    },
    OutputStatement {
        exprs: Vec<BoundNode>,
    },
    ReturnStatement {
        expr: Option<Box<BoundNode>>,
    },
    IfStatement {
        condition: Box<BoundNode>,
        block: Box<BoundNode>,
        else_block: Option<Box<BoundNode>>,
    },
    FunctionDeclaration {
        symbol: FunctionSymbol,
        block: Arc<BoundNode>,
    },
    ForLoop {
        iterator: VariableSymbol,
        lower_bound: Box<BoundNode>,
        upper_bound: Box<BoundNode>,
        block: Arc<BoundNode>,
    },
    WhileLoop {
        expr: Box<BoundNode>,
        block: Arc<BoundNode>,
    },
    UntilLoop {
        expr: Box<BoundNode>,
        block: Arc<BoundNode>,
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
        symbol: VariableSymbol,
        value: Box<BoundNode>,
    },
    BoundCallExpression {
        symbol: FunctionSymbol,
        args: Box<Vec<BoundNode>>,
    },
    ObjectExpression,
    ObjectMemberExpression {
        base: Box<BoundNode>,
        next: Box<BoundNode>,
    },
    IndexExpression {
        base: Box<BoundNode>,
        index: Box<BoundNode>,
    },
    IndexAssignmentExpression {
        base: Box<BoundNode>,
        index: Box<BoundNode>,
        value: Box<BoundNode>,
    },
    ReferenceExpression(VariableSymbol),
    NumberLiteral(i64),
    BooleanLiteral(bool),
    StringLiteral(String),
}

#[derive(Debug, Clone)]
pub struct BoundParameter {
    pub symbol: VariableSymbol,
    pub param_type: TypeKind,
}
