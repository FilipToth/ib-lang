use super::{bound_scope::BoundScope, types::TypeKind};

pub fn declare_builtin_functions(scope: &mut BoundScope) {
    scope.declare_function("input".to_string(), Vec::new(), TypeKind::String);
}
