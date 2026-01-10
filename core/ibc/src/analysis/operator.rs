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
    Modulo,
    IntDivision,
    Not,
    Equality,
    Inequality,
    LesserThan,
    LesserThanEquals,
    GreaterThan,
    GreaterThanEquals,
    And,
    Or,
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

        // whether type checking is disabled for this operation.
        let disabled = lhs_type == TypeKind::Any || rhs_type == TypeKind::Any;

        let span = rhs.span.clone();
        match self {
            Operator::Subtraction
            | Operator::Multiplication
            | Operator::Division
            | Operator::Modulo
            | Operator::IntDivision => {
                let ret = Some(TypeKind::Int);
                if disabled {
                    return ret;
                }

                if rhs_type != TypeKind::Int || lhs_type != TypeKind::Int {
                    let err = ErrorKind::BinaryOPeratorNotDefinedOnType {
                        op: self.clone(),
                        lhs: lhs_type,
                        rhs: rhs_type,
                    };

                    errors.add(err, span);
                    return None;
                }

                ret
            }
            Operator::Addition => {
                if disabled {
                    return Some(TypeKind::Any);
                }

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
            Operator::Equality | Operator::Inequality => {
                let ret = Some(TypeKind::Boolean);
                if disabled {
                    return ret;
                }

                if rhs_type != lhs_type {
                    let err = ErrorKind::EqualityNonMatchingTypes {
                        lhs: lhs_type,
                        rhs: rhs_type,
                    };

                    errors.add(err, span);
                    return None;
                }

                ret
            }
            Operator::LesserThan
            | Operator::LesserThanEquals
            | Operator::GreaterThan
            | Operator::GreaterThanEquals => {
                let ret = Some(TypeKind::Boolean);
                if disabled {
                    return ret;
                }

                if rhs_type != TypeKind::Int || lhs_type != TypeKind::Int {
                    let err = ErrorKind::BinaryOPeratorNotDefinedOnType {
                        op: self.clone(),
                        lhs: lhs_type,
                        rhs: rhs_type,
                    };

                    errors.add(err, span);
                    return None;
                }

                ret
            }
            Operator::And | Operator::Or => {
                let ret = Some(TypeKind::Boolean);
                if disabled {
                    return ret;
                }

                if rhs_type != TypeKind::Boolean || lhs_type != TypeKind::Boolean {
                    let err = ErrorKind::BinaryOPeratorNotDefinedOnType {
                        op: self.clone(),
                        lhs: lhs_type,
                        rhs: rhs_type,
                    };

                    errors.add(err, span);
                    return None;
                }

                ret
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
            Operator::Modulo => "mod",
            Operator::IntDivision => "div",
            Operator::Not => "NOT",
            Operator::Equality => "==",
            Operator::Inequality => "!=",
            Operator::LesserThan => "<",
            Operator::LesserThanEquals => "<=",
            Operator::GreaterThan => ">",
            Operator::GreaterThanEquals => ">=",
            Operator::And => "AND",
            Operator::Or => "OR",
        };

        op.to_string()
    }
}
