use super::{bound_node::BoundParameter, types::TypeKind};

#[derive(Debug)]
pub struct GlobalSymbolScope {
    num_symbols: u64,
}

impl GlobalSymbolScope {
    pub fn new() -> GlobalSymbolScope {
        GlobalSymbolScope { num_symbols: 0 }
    }

    pub fn alloc_variable(&mut self, identifier: String, var_type: TypeKind) -> VariableSymbol {
        self.num_symbols += 1;

        VariableSymbol {
            identifier: identifier,
            var_type: var_type,
            symbol_id: self.num_symbols,
        }
    }

    pub fn alloc_function(
        &mut self,
        identifier: String,
        parameters: Vec<BoundParameter>,
        ret_type: TypeKind,
    ) -> FunctionSymbol {
        self.num_symbols += 1;

        FunctionSymbol {
            identifier: identifier,
            parameters: parameters,
            ret_type: ret_type,
            symbol_id: self.num_symbols,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VariableSymbol {
    pub identifier: String,
    pub var_type: TypeKind,
    pub symbol_id: u64,
}

#[derive(Debug, Clone)]
pub struct FunctionSymbol {
    pub identifier: String,
    pub parameters: Vec<BoundParameter>,
    pub ret_type: TypeKind,
    pub symbol_id: u64,
}
