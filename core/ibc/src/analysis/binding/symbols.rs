use std::collections::HashMap;

use super::{bound_node::BoundParameter, types::TypeKind};

#[derive(Debug)]
pub struct GlobalSymbolScope {
    num_symbols: u64,
    /// The functions whose bodies are being bound, innermost last.
    binding: Vec<u64>,
    /// The variables allocated inside each function's body, by function id.
    ///
    /// Every scope shares this allocator, so a variable declared in a nested
    /// block of a function body is recorded here too. A call gives these their
    /// own storage, which is what lets a function call itself; anything
    /// declared outside a function is absent, so assigning it from within one
    /// still changes the one variable everybody sees.
    locals: HashMap<u64, Vec<u64>>,
}

impl GlobalSymbolScope {
    pub fn new() -> GlobalSymbolScope {
        GlobalSymbolScope {
            num_symbols: 0,
            binding: Vec::new(),
            locals: HashMap::new(),
        }
    }

    pub fn enter_function(&mut self, symbol_id: u64) {
        self.binding.push(symbol_id);
    }

    pub fn exit_function(&mut self) {
        self.binding.pop();
    }

    pub fn locals_of(&self, symbol_id: u64) -> Vec<u64> {
        match self.locals.get(&symbol_id) {
            Some(locals) => locals.clone(),
            None => Vec::new(),
        }
    }

    pub fn alloc_variable(&mut self, identifier: String, var_type: TypeKind) -> VariableSymbol {
        self.num_symbols += 1;

        if let Some(function) = self.binding.last() {
            let locals = self.locals.entry(*function).or_default();
            locals.push(self.num_symbols);
        }

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
