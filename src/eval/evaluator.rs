use std::{collections::HashMap, mem};

use crate::analysis::{
    binding::{
        bound_node::{BoundNode, BoundNodeKind},
        symbols::VariableSymbol,
    },
    operator::Operator,
};

pub struct EvalHeap {
    // just use rust's heap to manage
    // memory, no need for us to make
    // our own heap
    variables: HashMap<u64, EvalValue>,
}

impl EvalHeap {
    fn new() -> EvalHeap {
        EvalHeap {
            variables: HashMap::new(),
        }
    }

    fn assign_var(&mut self, symbol: &VariableSymbol, val: EvalValue) {
        let id = symbol.symbol_id;
        self.variables.insert(id, val);
    }

    fn get_var(&self, symbol: &VariableSymbol) -> EvalValue {
        let id = &symbol.symbol_id;
        let value = &self.variables[id];
        value.clone()
    }
}

#[derive(Debug, Clone)]
pub enum EvalValue {
    Void,
    Int(i32),
    Bool(bool),
    String(String),
}

impl EvalValue {
    fn void() -> EvalValue {
        EvalValue::Void
    }

    fn int(val: i32) -> EvalValue {
        EvalValue::Int(val)
    }

    fn bool(val: bool) -> EvalValue {
        EvalValue::Bool(val)
    }

    fn string(val: String) -> EvalValue {
        EvalValue::String(val)
    }

    pub fn to_string(&self) -> String {
        match self {
            EvalValue::Void => "void".to_string(),
            EvalValue::Int(val) => val.to_string(),
            EvalValue::Bool(val) => val.to_string(),
            EvalValue::String(val) => val.clone(),
        }
    }

    fn force_get_int(&self) -> i32 {
        let EvalValue::Int(val) = self else {
            unreachable!()
        };

        val.clone()
    }

    fn force_get_bool(&self) -> bool {
        let EvalValue::Bool(val) = self else {
            unreachable!()
        };

        val.clone()
    }

    fn force_get_string(&self) -> String {
        let EvalValue::String(val) = self else {
            unreachable!()
        };

        val.clone()
    }
}

fn eval_binary_expr(lhs: EvalValue, op: &Operator, rhs: EvalValue) -> EvalValue {
    match op {
        Operator::Addition => {
            // addition is only defined on integers
            let lhs = lhs.force_get_int();
            let rhs = rhs.force_get_int();
            EvalValue::int(lhs + rhs)
        }
        Operator::Subtraction => {
            // subtraction is only defined on integers
            let lhs = lhs.force_get_int();
            let rhs = rhs.force_get_int();
            EvalValue::int(lhs - rhs)
        }
        Operator::Multiplication => {
            // multiplication is only defined on integers
            let lhs = lhs.force_get_int();
            let rhs = rhs.force_get_int();
            EvalValue::int(lhs * rhs)
        }
        Operator::Division => {
            // division is only defined on integers
            let lhs = lhs.force_get_int();
            let rhs = rhs.force_get_int();
            EvalValue::int(lhs / rhs)
        }
        Operator::Equality => {
            // check if same variant
            if mem::discriminant(&lhs) != mem::discriminant(&rhs) {
                unreachable!();
            }

            match lhs {
                EvalValue::Void => unreachable!(),
                EvalValue::Int(lhs) => {
                    let rhs = rhs.force_get_int();
                    EvalValue::Bool(rhs == lhs)
                }
                EvalValue::Bool(lhs) => {
                    let rhs = rhs.force_get_bool();
                    EvalValue::Bool(rhs == lhs)
                }
                EvalValue::String(lhs) => {
                    let rhs = rhs.force_get_string();
                    EvalValue::Bool(rhs == lhs)
                }
            }
        }
        _ => {
            unreachable!("Not a binary operator")
        }
    }
}

fn eval_unary_expr(rhs_val: EvalValue, op: &Operator) -> EvalValue {
    match op {
        Operator::Not => {
            // only defined on bools
            let rhs = rhs_val.force_get_bool();
            EvalValue::Bool(!rhs)
        }
        _ => {
            unreachable!("Not a unary operator")
        }
    }
}

fn eval_rec(node: &BoundNode, heap: &mut EvalHeap) -> EvalValue {
    let val = match &node.kind {
        BoundNodeKind::Module { block } => eval_rec(&block, heap),
        BoundNodeKind::Block { children } => {
            for child in children.iter() {
                eval_rec(child, heap);
            }

            EvalValue::void()
        }
        BoundNodeKind::AssignmentExpression { symbol, value } => {
            let value = eval_rec(&value, heap);
            heap.assign_var(symbol, value.clone());

            value
        }
        BoundNodeKind::ReferenceExpression(reference) => heap.get_var(&reference),
        BoundNodeKind::BinaryExpression { lhs, op, rhs } => {
            let lhs_val = eval_rec(&lhs, heap);
            let rhs_val = eval_rec(&rhs, heap);
            eval_binary_expr(lhs_val, op, rhs_val)
        }
        BoundNodeKind::UnaryExpression { op, rhs } => {
            let rhs_val = eval_rec(&rhs, heap);
            eval_unary_expr(rhs_val, op)
        }
        BoundNodeKind::NumberLiteral(num) => EvalValue::int(*num),
        BoundNodeKind::BooleanLiteral(val) => EvalValue::bool(*val),
        BoundNodeKind::OutputStatement { expr } => {
            let value = eval_rec(&expr, heap);
            println!("{}", value.to_string());

            EvalValue::void()
        }
        BoundNodeKind::ReturnStatement { expr } => todo!(),
        BoundNodeKind::IfStatement { condition, block, else_block } => {
            let cond_value = eval_rec(&condition, heap).force_get_bool();
            if cond_value {
                eval_rec(&block, heap);
            } else if let Some(else_block) = else_block {
                eval_rec(else_block, heap);
            }

            EvalValue::void()
        },
        BoundNodeKind::FunctionDeclaration { symbol, block } => todo!(),
        BoundNodeKind::BoundCallExpression { symbol, args } => todo!(),
    };

    val
}

pub fn eval(root: &BoundNode) {
    let mut heap = EvalHeap::new();
    eval_rec(root, &mut heap);
}
