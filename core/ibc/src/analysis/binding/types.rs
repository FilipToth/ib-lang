use crate::{
    analysis::{
        error_bag::{ErrorBag, ErrorKind},
        syntax::syntax_token::TypeAnnotation,
    },
    eval::evaluator::EvalValue,
};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum TypeKind {
    Any,
    Void,
    Int,
    String,
    Boolean,
    Array(Box<TypeKind>),
    Collection(Box<TypeKind>),
    Stack(Box<TypeKind>),
    Queue(Box<TypeKind>),
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
            TypeKind::Any => "Any".to_string(),
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
            TypeKind::Stack(generic) => {
                let generic = generic.to_string();
                format!("Stack<{}>", generic)
            }
            TypeKind::Queue(generic) => {
                let generic = generic.to_string();
                format!("Queue<{}>", generic)
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

                let is_empty = TypeMethodRepresentation {
                    identifier: "isEmpty".to_string(),
                    ret_type: TypeKind::Boolean,
                    params: Vec::new(),
                };

                methods.push(add);
                methods.push(get);
                methods.push(len);
                methods.push(is_empty);
            }
            TypeKind::Collection(generic) => {
                let generic = *generic.clone();
                let has_next = TypeMethodRepresentation {
                    identifier: "hasNext".to_string(),
                    ret_type: TypeKind::Boolean,
                    params: Vec::new(),
                };

                let get_item = TypeMethodRepresentation {
                    identifier: "getNext".to_string(),
                    ret_type: generic.clone(),
                    params: Vec::new(),
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
            TypeKind::Stack(generic) => {
                let generic = *generic.clone();
                let push = TypeMethodRepresentation {
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

                let pop = TypeMethodRepresentation {
                    identifier: "pop".to_string(),
                    ret_type: generic,
                    params: Vec::new(),
                };

                let is_empty = TypeMethodRepresentation {
                    identifier: "isEmpty".to_string(),
                    ret_type: TypeKind::Boolean,
                    params: Vec::new(),
                };

                methods.push(push);
                methods.push(pop);
                methods.push(is_empty);
            }
            TypeKind::Queue(generic) => {
                let generic = *generic.clone();
                let enqueue = TypeMethodRepresentation {
                    identifier: "enqueue".to_string(),
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

                let dequeue = TypeMethodRepresentation {
                    identifier: "dequeue".to_string(),
                    ret_type: generic,
                    params: Vec::new(),
                };

                let is_empty = TypeMethodRepresentation {
                    identifier: "isEmpty".to_string(),
                    ret_type: TypeKind::Boolean,
                    params: Vec::new(),
                };

                methods.push(enqueue);
                methods.push(dequeue);
                methods.push(is_empty);
            }
            _ => {}
        }

        methods
    }
}

pub fn get_type(annotation: &TypeAnnotation, errors: &mut ErrorBag) -> Option<TypeKind> {
    let type_kind = match annotation.name.as_str() {
        "Any" => TypeKind::Any,
        "Void" => TypeKind::Void,
        "Int" => TypeKind::Int,
        "String" => TypeKind::String,
        "Boolean" => TypeKind::Boolean,
        "Array" => TypeKind::Array(Box::new(get_generic(annotation, errors)?)),
        "Collection" => TypeKind::Collection(Box::new(get_generic(annotation, errors)?)),
        "Stack" => TypeKind::Stack(Box::new(get_generic(annotation, errors)?)),
        "Queue" => TypeKind::Queue(Box::new(get_generic(annotation, errors)?)),
        _ => {
            let kind = ErrorKind::UndefinedType(annotation.name.clone());
            errors.add(kind, annotation.span);
            return None;
        }
    };

    Some(type_kind)
}

/// Resolves the element type a collection requires. It is a full type, so
/// collections of collections resolve recursively.
fn get_generic(annotation: &TypeAnnotation, errors: &mut ErrorBag) -> Option<TypeKind> {
    match &annotation.generic {
        Some(generic) => get_type(generic, errors),
        None => {
            let kind = ErrorKind::ExpectsGenericTypeParam(annotation.name.clone());
            errors.add(kind, annotation.span);
            None
        }
    }
}

#[derive(Debug, Clone)]
pub enum ObjectState {
    Array(ArrayState),
    Collection(CollectionState),
    Stack(ArrayState),
    Queue(ArrayState),
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
        TypeKind::Stack(_) => ObjectState::Stack(ArrayState::new()),
        TypeKind::Queue(_) => ObjectState::Queue(ArrayState::new()),
        _ => unreachable!(),
    }
}
