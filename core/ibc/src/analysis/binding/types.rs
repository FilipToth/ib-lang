use crate::{
    analysis::{
        error_bag::{ErrorBag, ErrorKind},
        span::Span,
    },
    eval::evaluator::EvalValue,
};

use super::bound_node::BoundParameter;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum TypeKind {
    Void,
    Int,
    String,
    Boolean,
    Array(Box<TypeKind>),
}

pub struct TypeMethodRepresentation {
    pub identifier: String,
    pub ret_type: TypeKind,
    pub params: Vec<TypeMethodParamRepresentation>,
}

pub struct TypeMethodParamRepresentation {
    pub identifier: String,
    pub param_type: TypeKind,
}

impl TypeKind {
    pub fn to_string(&self) -> String {
        match &self {
            TypeKind::Void => "Void".to_string(),
            TypeKind::Int => "Int".to_string(),
            TypeKind::String => "String".to_string(),
            TypeKind::Boolean => "Boolean".to_string(),
            TypeKind::Array(generic) => {
                let generic = generic.to_string();
                format!("Array<{}>", generic)
            }
        }
    }

    pub fn reflection_methods(&self) -> Vec<TypeMethodRepresentation> {
        let mut methods: Vec<TypeMethodRepresentation> = Vec::new();

        match &self {
            TypeKind::Array(generic) => {
                let generic = *generic.clone();
                let add = TypeMethodRepresentation {
                    identifier: "push".to_string(),
                    ret_type: TypeKind::Void,
                    params: {
                        let mut params = Vec::<TypeMethodParamRepresentation>::new();
                        let item = TypeMethodParamRepresentation {
                            identifier: "item".to_string(),
                            param_type: generic.clone(),
                        };

                        params.push(item);
                        params
                    },
                };

                let get = TypeMethodRepresentation {
                    identifier: "get".to_string(),
                    ret_type: generic,
                    params: {
                        let mut params = Vec::<TypeMethodParamRepresentation>::new();
                        let item = TypeMethodParamRepresentation {
                            identifier: "index".to_string(),
                            param_type: TypeKind::Int,
                        };

                        params.push(item);
                        params
                    },
                };

                let len = TypeMethodRepresentation {
                    identifier: "len".to_string(),
                    ret_type: TypeKind::Int,
                    params: Vec::new(),
                };

                methods.push(add);
                methods.push(get);
                methods.push(len);
            }
            _ => {}
        }

        methods
    }
}

pub fn get_type(
    identifier: String,
    generic: Option<String>,
    span: &Span,
    errors: &mut ErrorBag,
) -> Option<TypeKind> {
    let type_kind = match identifier.as_str() {
        "Void" => TypeKind::Void,
        "Int" => TypeKind::Int,
        "String" => TypeKind::String,
        "Boolean" => TypeKind::Boolean,
        "Array" => {
            let generic = match generic {
                Some(id) => match get_type(id, None, span, errors) {
                    Some(t) => t,
                    None => return None,
                },
                None => {
                    let kind = ErrorKind::ExpectsGenericTypeParam("Array".to_string());
                    errors.add(kind, span.clone());
                    return None;
                }
            };

            TypeKind::Array(Box::new(generic))
        }
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
    pub internal: Vec<EvalValue>,
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
        TypeKind::Array(generic) => ObjectState::Array(ArrayState::new()),
        _ => unreachable!(),
    }
}
