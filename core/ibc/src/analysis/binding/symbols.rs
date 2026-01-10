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
    /// Unique within the scope that declared it. For type-method parameters
    /// this is an internal mangled name -- use `name()` to display it.
    pub identifier: String,
    pub var_type: TypeKind,
    pub symbol_id: u64,
}

impl VariableSymbol {
    /// The name the user wrote.
    ///
    /// Type-method parameters are declared under a mangled identifier of the
    /// form `$param$<type>$<method>$<name>`, so the user's name is whatever
    /// follows the last `$`. Everything else is already the user's name, and
    /// `$` cannot appear in one, so the split is unambiguous.
    ///
    /// For diagnostics and analytics only -- the evaluator keys off
    /// `symbol_id` and never needs this.
    pub fn name(&self) -> &str {
        if !self.identifier.starts_with('$') {
            return &self.identifier;
        }

        match self.identifier.rfind('$') {
            Some(index) => &self.identifier[index + 1..],
            None => &self.identifier,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FunctionSymbol {
    pub identifier: String,
    pub parameters: Vec<BoundParameter>,
    pub ret_type: TypeKind,
    pub symbol_id: u64,
}
