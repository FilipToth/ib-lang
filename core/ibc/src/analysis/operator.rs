use super::{
    binding::{bound_node::BoundNode, types::TypeKind},
    error_bag::{ErrorBag, ErrorKind},
};

#[derive(Debug, Clone)]
pub enum Operator {
    Addition,
    Subtraction,
    Division,
    Multiplication,
    Not,
    Equality,
    LesserThan,
    GreaterThan,
}

impl Operator {
    pub fn return_type_unary(&self, rhs: &BoundNode, errors: &mut ErrorBag) -> Option<TypeKind> {
        let span = rhs.span.clone();
        let rhs_type = rhs.node_type.clone();
        match self {
            Operator::Not => {
                if rhs_type != TypeKind::Boolean {
                    let err = ErrorKind::UnaryOperatorNotDefinedOnType {
                        op: self.clone(),
                        used_type: rhs_type,
                    };

                    errors.add(err, span);
                    return None;
                }

                return Some(TypeKind::Boolean);
            }
            Operator::Subtraction => {
                if rhs_type != TypeKind::Int {
                    let err = ErrorKind::UnaryOperatorNotDefinedOnType {
                        op: self.clone(),
                        used_type: rhs_type,
                    };

                    errors.add(err, span);
                    return None;
                }

                return Some(TypeKind::Int);
            }
            _ => unreachable!("Operator {:?}", self),
        }
    }

    pub fn return_type_binary(
        &self,
        lhs: &BoundNode,
        rhs: &BoundNode,
        errors: &mut ErrorBag,
    ) -> Option<TypeKind> {
        let rhs_type = rhs.node_type.clone();
        let lhs_type = lhs.node_type.clone();

        let span = rhs.span.clone();
        match self {
            Operator::Subtraction | Operator::Multiplication | Operator::Division => {
                if rhs_type != TypeKind::Int || lhs_type != TypeKind::Int {
                    let err = ErrorKind::BinaryOPeratorNotDefinedOnType {
                        op: self.clone(),
                        lhs: lhs_type,
                        rhs: rhs_type,
                    };

                    errors.add(err, span);
                    return None;
                }

                return Some(TypeKind::Int);
            }
            Operator::Addition => {
                if rhs_type == TypeKind::String || lhs_type == TypeKind::String {
                    return Some(TypeKind::String);
                } else if rhs_type == TypeKind::Int && lhs_type == TypeKind::Int {
                    return Some(TypeKind::Int);
                }

                let err = ErrorKind::BinaryOPeratorNotDefinedOnType {
                    op: self.clone(),
                    lhs: lhs_type,
                    rhs: rhs_type,
                };

                errors.add(err, span);
                None
            }
            Operator::Equality => {
                if rhs_type != lhs_type {
                    let err = ErrorKind::EqualityNonMatchingTypes {
                        lhs: lhs_type,
                        rhs: rhs_type,
                    };

                    errors.add(err, span);
                    return None;
                }

                Some(TypeKind::Boolean)
            }
            Operator::LesserThan | Operator::GreaterThan => {
                if rhs_type != TypeKind::Int || lhs_type != TypeKind::Int {
                    let err = ErrorKind::BinaryOPeratorNotDefinedOnType {
                        op: self.clone(),
                        lhs: lhs_type,
                        rhs: rhs_type,
                    };

                    errors.add(err, span);
                    return None;
                }

                return Some(TypeKind::Boolean);
            }
            _ => unreachable!(),
        }
    }

    pub fn to_string(&self) -> String {
        let op = match &self {
            Operator::Addition => "+",
            Operator::Subtraction => "-",
            Operator::Division => "/",
            Operator::Multiplication => "*",
            Operator::Not => "!",
            Operator::Equality => "==",
            Operator::LesserThan => "<",
            Operator::GreaterThan => ">",
        };

        op.to_string()
    }
}
