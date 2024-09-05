use std::{collections::HashMap, mem, rc::Rc};

use crate::analysis::{
    binding::{
        bound_node::{BoundNode, BoundNodeKind},
        symbols::{FunctionSymbol, VariableSymbol},
    },
    operator::Operator,
};

pub struct EvalInfo<'a> {
    heap: &'a mut EvalHeap,
    output: &'a mut String
}

pub struct EvalHeap {
    // just use rust's heap to manage
    // memory, no need for us to make
    // our own heap
    variables: HashMap<u64, EvalValue>,
    functions: HashMap<u64, Rc<BoundNode>>
}

impl EvalHeap {
    fn new() -> EvalHeap {
        EvalHeap {
            variables: HashMap::new(),
            functions: HashMap::new(),
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

    fn declare_func(&mut self, symbol: &FunctionSymbol, body: Rc<BoundNode>) {
        let id = symbol.symbol_id;
        self.functions.insert(id, body);
    }

    fn get_func(&self, symbol: &FunctionSymbol) -> Rc<BoundNode> {
        let id = &symbol.symbol_id;
        let body = &self.functions[id];
        body.clone()
    }
}

#[derive(Debug, Clone)]
pub enum EvalValue {
    Void,
    Int(i32),
    Bool(bool),
    String(String),
    // used to return in
    // the eval rec function
    Return(Box<EvalValue>),
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
            EvalValue::Return(_) => unreachable!(),
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
                },
                EvalValue::Return(_) => unreachable!()
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

fn eval_rec(node: &BoundNode, info: &mut EvalInfo) -> EvalValue {
    let val = match &node.kind {
        BoundNodeKind::Module { block } => eval_rec(&block, info),
        BoundNodeKind::Block { children } => {
            for child in children.iter() {
                let val = eval_rec(child, info);
                if let EvalValue::Return(_) = &val {
                    return val;
                }
            }

            EvalValue::void()
        }
        BoundNodeKind::AssignmentExpression { symbol, value } => {
            let value = eval_rec(&value, info);
            info.heap.assign_var(symbol, value.clone());

            value
        }
        BoundNodeKind::ReferenceExpression(reference) => info.heap.get_var(&reference),
        BoundNodeKind::BinaryExpression { lhs, op, rhs } => {
            let lhs_val = eval_rec(&lhs, info);
            let rhs_val = eval_rec(&rhs, info);
            eval_binary_expr(lhs_val, op, rhs_val)
        }
        BoundNodeKind::UnaryExpression { op, rhs } => {
            let rhs_val = eval_rec(&rhs, info);
            eval_unary_expr(rhs_val, op)
        }
        BoundNodeKind::NumberLiteral(num) => EvalValue::int(*num),
        BoundNodeKind::BooleanLiteral(val) => EvalValue::bool(*val),
        BoundNodeKind::OutputStatement { expr } => {
            let value = eval_rec(&expr, info);

            let value = format!("{}\n", value.to_string());
            String::push_str(info.output, &value);

            EvalValue::void()
        }
        BoundNodeKind::ReturnStatement { expr } => {
            // create special return value
            let val = if let Some(expr) = expr {
                eval_rec(&expr, info)
            } else {
                EvalValue::void()
            };

            EvalValue::Return(Box::new(val))
        },
        BoundNodeKind::IfStatement { condition, block, else_block } => {
            let cond_value = eval_rec(&condition, info).force_get_bool();
            let value = if cond_value {
                eval_rec(&block, info)
            } else if let Some(else_block) = else_block {
                eval_rec(else_block, info)
            } else {
                EvalValue::void()
            };

            match &value {
                EvalValue::Return(_) => value,
                _ => EvalValue::void()
            }
        },
        BoundNodeKind::FunctionDeclaration { symbol, block } => {
            info.heap.declare_func(symbol, block.clone());
            EvalValue::void()
        },
        BoundNodeKind::BoundCallExpression { symbol, args } => {
            let num_params = symbol.parameters.len();
            for index in 0..num_params {
                let param = &symbol.parameters[index];
                let arg = &args[index];

                let symbol = &param.symbol;
                let value = eval_rec(arg, info);
                info.heap.assign_var(symbol, value);
            }

            // no need to clear arguments after executing the block
            let body = info.heap.get_func(symbol);
            let ret_value = eval_rec(&body, info);

            // unwrap from return
            let EvalValue::Return(ret_value) = ret_value else {
                unreachable!();
            };

            ret_value.as_ref().clone()
        },
    };

    val
}

pub fn eval(root: &BoundNode, output: &mut String) {
    let mut heap = EvalHeap::new();
    let mut info = EvalInfo {
        heap: &mut heap,
        output: output
    };

    eval_rec(root, &mut info);
}
