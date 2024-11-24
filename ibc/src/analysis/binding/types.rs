use crate::{
    analysis::{
        error_bag::{ErrorBag, ErrorKind},
        span::Span,
    },
    eval::evaluator::EvalValue,
};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum TypeKind {
    Void,
    Int,
    String,
    Boolean,
    Array,
}

impl TypeKind {
    pub fn to_string(&self) -> String {
        let kind = match &self {
            TypeKind::Void => "Void",
            TypeKind::Int => "Int",
            TypeKind::String => "String",
            TypeKind::Boolean => "Boolean",
            TypeKind::Array => "Array",
        };

        kind.to_string()
    }
}

pub fn get_type(identifier: String, span: &Span, errors: &mut ErrorBag) -> Option<TypeKind> {
    let type_kind = match identifier.as_str() {
        "Void" => TypeKind::Void,
        "Int" => TypeKind::Int,
        "String" => TypeKind::String,
        "Boolean" => TypeKind::Boolean,
        "Array" => TypeKind::Array,
        _ => {
            let kind = ErrorKind::UndefinedType(identifier);
            errors.add(kind, span.clone());
            return None;
        }
    };

    Some(type_kind)
}

#[derive(Debug, Clone)]
pub enum ObjectState {
    Array(ArrayState),
}

pub trait TypeObject {
    fn execute_method();
    fn get_value();
}

#[derive(Debug, Clone)]
pub struct ArrayState {
    internal: Vec<EvalValue>,
}

impl TypeObject for ArrayState {
    fn execute_method() {
        todo!()
    }

    fn get_value() {
        todo!()
    }
}

impl ArrayState {
    fn new() -> Self {
        ArrayState {
            internal: Vec::new(),
        }
    }
}

pub fn get_object_state(tp: TypeKind) -> ObjectState {
    match tp {
        TypeKind::Array => ObjectState::Array(ArrayState::new()),
        _ => unreachable!(),
    }
}
