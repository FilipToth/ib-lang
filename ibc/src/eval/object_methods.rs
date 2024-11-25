use std::{cell::RefCell, rc::Rc};

use crate::analysis::binding::{
    symbols::FunctionSymbol,
    types::{ArrayState, ObjectState},
};

use super::evaluator::{EvalInfo, EvalValue};

fn execute_array_method(
    state: &mut ArrayState,
    symbol: &FunctionSymbol,
    info: &mut EvalInfo,
) -> EvalValue {
    match symbol.identifier.as_str() {
        "push" => {
            let item = &symbol.parameters[0].symbol;
            let item_value = info.heap.get_var(item);

            state.internal.push(item_value);
            EvalValue::Void
        }
        "get" => {
            let index = &symbol.parameters[0].symbol;
            let index_value = info.heap.get_var(index);

            let index_value = match index_value {
                EvalValue::Int(i) => i as usize,
                _ => unreachable!(),
            };

            let value = &state.internal[index_value];
            value.clone()
        }
        "len" => {
            let length = state.internal.len() as i64;
            EvalValue::Int(length)
        }
        _ => unimplemented!(),
    }
}

fn execute_object_method(
    state: Rc<RefCell<ObjectState>>,
    symbol: &FunctionSymbol,
    info: &mut EvalInfo,
) -> EvalValue {
    let mut state = state.borrow_mut();
    match &mut *state {
        ObjectState::Array(state) => execute_array_method(state, symbol, info),
    }
}

pub fn execute_type_method(
    mut value: EvalValue,
    symbol: &FunctionSymbol,
    info: &mut EvalInfo,
) -> EvalValue {
    match &mut value {
        EvalValue::Object(state) => execute_object_method(state.clone(), symbol, info),
        _ => unimplemented!(),
    }
}
