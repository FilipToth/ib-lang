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
    Array(Box<TypeKind>),
    Collection(Box<TypeKind>),
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
            TypeKind::Collection(generic) => {
                let generic = generic.to_string();
                format!("Collection<{}>", generic)
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
            TypeKind::Collection(generic) => {
                let generic = *generic.clone();
                let has_next = TypeMethodRepresentation {
                    identifier: "hasNext".to_string(),
                    ret_type: TypeKind::Boolean,
                    params: Vec::new()
                };

                let get_item = TypeMethodRepresentation {
                    identifier: "getItem".to_string(),
                    ret_type: generic.clone(),
                    params: Vec::new()
                };

                let reset_next = TypeMethodRepresentation {
                    identifier: "resetNext".to_string(),
                    ret_type: TypeKind::Void,
                    params: Vec::new(),
                };

                let add_item = TypeMethodRepresentation {
                    identifier: "addItem".to_string(),
                    ret_type: TypeKind::Void,
                    params: {
                        let mut params = Vec::<TypeMethodParamRepresentation>::new();
                        let item = TypeMethodParamRepresentation {
                            identifier: "item".to_string(),
                            param_type: generic,
                        };

                        params.push(item);
                        params
                    },
                };

                let is_empty = TypeMethodRepresentation {
                    identifier: "isEmpty".to_string(),
                    ret_type: TypeKind::Boolean,
                    params: Vec::new(),
                };

                methods.push(has_next);
                methods.push(get_item);
                methods.push(reset_next);
                methods.push(add_item);
                methods.push(is_empty);
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
        "Collection" => {
            let generic = match generic {
                Some(id) => match get_type(id, None, span, errors) {
                    Some(t) => t,
                    None => return None,
                },
                None => {
                    let kind = ErrorKind::ExpectsGenericTypeParam("Collection".to_string());
                    errors.add(kind, span.clone());
                    return None;
                }
            };

            TypeKind::Collection(Box::new(generic))
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
    Collection(CollectionState),
}

#[derive(Debug, Clone)]
pub struct ArrayState {
    pub internal: Vec<EvalValue>,
}

impl ArrayState {
    fn new() -> Self {
        ArrayState {
            internal: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CollectionState {
    pub head: usize,
    pub internal: Vec<EvalValue>,
}

impl CollectionState {
    fn new() -> Self {
        CollectionState {
            head: 0,
            internal: Vec::new(),
        }
    }
}

pub fn get_object_state(tp: TypeKind) -> ObjectState {
    match tp {
        TypeKind::Array(_) => ObjectState::Array(ArrayState::new()),
        TypeKind::Collection(_) => ObjectState::Collection(CollectionState::new()),
        _ => unreachable!(),
    }
}
