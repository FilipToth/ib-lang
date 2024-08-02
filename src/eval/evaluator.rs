use std::collections::HashMap;

use crate::analysis::binding::{bound_node::{BoundNode, BoundNodeKind}, symbols::VariableSymbol};

pub struct EvalHeap {
    // just use rust's heap to manage
    // memory, no need for us to make
    // our own heap
    variables: HashMap<u64, EvalValue>
}

impl EvalHeap {
    fn new() -> EvalHeap {
        EvalHeap { variables: HashMap::new() }
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
pub struct EvalValue {
    // yes, this is INCREDIBLY
    // memory inefficient, but
    // it's what you get for now
    is_void: bool,
    int_val: Option<i32>,
    bool_val: Option<bool>,
    string_val: Option<String>,
}

impl EvalValue {
    fn void() -> EvalValue {
        EvalValue { is_void: true, int_val: None, bool_val: None, string_val: None }
    }

    fn int(val: i32) -> EvalValue {
        EvalValue { is_void: false, int_val: Some(val), bool_val: None, string_val: None }
    }

    fn bool(val: bool) -> EvalValue {
        EvalValue { is_void: false, int_val: None, bool_val: Some(val), string_val: None }
    }

    fn string(val: String) -> EvalValue {
        EvalValue { is_void: false, int_val: None, bool_val: None, string_val: Some(val) }
    }

    pub fn to_string(&self) -> String {
        if self.is_void {
            "void".to_string()
        } else if let Some(val) = self.int_val {
            val.to_string()
        } else if let Some(val) = self.bool_val {
            val.to_string()
        } else if let Some(val) = &self.string_val {
            val.clone()
        } else {
            unreachable!()
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
        },
        BoundNodeKind::AssignmentExpression { symbol, value } => {
            let value = eval_rec(&value, heap);
            heap.assign_var(symbol, value.clone());

            value
        },
        BoundNodeKind::ReferenceExpression(reference) => {
            heap.get_var(&reference)
        },
        BoundNodeKind::NumberLiteral(num) => EvalValue::int(*num),
        BoundNodeKind::BooleanLiteral(val) => EvalValue::bool(*val),
        BoundNodeKind::OutputStatement { expr } => {
            let value = eval_rec(&expr, heap);
            println!("{}", value.to_string());

            EvalValue::void()
        },
        _ => unreachable!(),
    };

    val
}

pub fn eval(root: &BoundNode) {
    let mut heap = EvalHeap::new();
    eval_rec(root, &mut heap);
}
