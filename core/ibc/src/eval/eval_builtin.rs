use crate::analysis::binding::symbols::FunctionSymbol;
use super::evaluator::{EvalInfo, EvalValue};

pub fn try_eval_builtin(symbol: &FunctionSymbol, info: &mut EvalInfo) -> Option<EvalValue> {
    match symbol.identifier.as_str() {
        "input" => {
            let input = (info.input_handler)();
            let val = EvalValue::String(input);
            Some(val)
        },
        _ => None
    }
}