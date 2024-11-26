use std::{cell::RefCell, rc::Rc};

use crate::analysis::binding::{
    symbols::FunctionSymbol,
    types::{ArrayState, CollectionState, ObjectState},
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

            match state.internal.get(index_value) {
                Some(v) => v.clone(),
                None => {
                    panic!("Runtime error");
                }
            }
        }
        "len" => {
            let length = state.internal.len() as i64;
            EvalValue::Int(length)
        }
        _ => unimplemented!(),
    }
}

fn execute_collection_method(
    state: &mut CollectionState,
    symbol: &FunctionSymbol,
    info: &mut EvalInfo,
) -> EvalValue {
    match symbol.identifier.as_str() {
        "hasNext" => {
            let index = state.head.clone();
            let res = state.internal.get(index).is_some();
            EvalValue::Bool(res)
        }
        "getItem" => {
            let index = state.head.clone();
            match state.internal.get(index) {
                Some(v) => {
                    state.head += 1;
                    v.clone()
                }
                None => {
                    panic!("Runtime error");
                }
            }
        }
        "resetNext" => {
            state.head = 0;
            EvalValue::Void
        }
        "addItem" => {
            let item = &symbol.parameters[0].symbol;
            let item_value = info.heap.get_var(item);

            state.internal.push(item_value);
            EvalValue::Void
        }
        "isEmpty" => {
            let res = state.internal.len() == 0;
            EvalValue::Bool(res)
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
        ObjectState::Collection(state) => execute_collection_method(state, symbol, info),
    }
}

pub fn eval_type_method(
    mut value: EvalValue,
    symbol: &FunctionSymbol,
    info: &mut EvalInfo,
) -> EvalValue {
    match &mut value {
        EvalValue::Object(state) => execute_object_method(state.clone(), symbol, info),
        _ => unimplemented!(),
    }
}
