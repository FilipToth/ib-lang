use crate::analysis::{
    error_bag::{ErrorBag, ErrorKind},
    CodeLocation,
};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum TypeKind {
    Void,
    Int,
    String,
    Boolean,
}

impl TypeKind {
    pub fn to_string(&self) -> String {
        let kind = match &self {
            TypeKind::Void => "Void",
            TypeKind::Int => "Int",
            TypeKind::String => "String",
            TypeKind::Boolean => "Boolean",
        };

        kind.to_string()
    }
}

pub fn get_type(identifier: String, loc: &CodeLocation, errors: &mut ErrorBag) -> Option<TypeKind> {
    let type_kind = match identifier.as_str() {
        "Void" => TypeKind::Void,
        "Int" => TypeKind::Int,
        "String" => TypeKind::String,
        "Boolean" => TypeKind::Boolean,
        _ => {
            let kind = ErrorKind::UndefinedType(identifier);
            errors.add(kind, loc.line, loc.col);
            return None;
        }
    };

    Some(type_kind)
}
