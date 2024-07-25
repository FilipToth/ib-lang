#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum TypeKind {
    Void,
    Int,
    String,
    Boolean,
}

pub fn get_type(identifier: String) -> Option<TypeKind> {
    let type_kind = match identifier.as_str() {
        "Void" => TypeKind::Void,
        "Int" => TypeKind::Int,
        "String" => TypeKind::String,
        "Boolean" => TypeKind::Boolean,
        _ => return None,
    };

    Some(type_kind)
}
