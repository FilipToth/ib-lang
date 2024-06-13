use super::{binding::types::TypeKind, operator::Operator};

pub enum ErrorKind {
    FailedParsing,
    NumberParsing,
    UnaryOperatorNotDefinedOnType {
        op: Operator,
        used_type: TypeKind,
    },
    BinaryOPeratorNotDefinedOnType {
        op: Operator,
        lhs: TypeKind,
        rhs: TypeKind,
    },
    EqualityNonMatchingTypes {
        lhs: TypeKind,
        rhs: TypeKind,
    },
}

impl ErrorKind {
    pub fn format(&self) -> String {
        match self {
            Self::FailedParsing => "Failed parsing".to_string(),
            Self::NumberParsing => "Cannot parse number".to_string(),
            Self::UnaryOperatorNotDefinedOnType { op, used_type } => {
                format!(
                    "Unary operator '{:?}' not defined on type: '{:?}'",
                    op, used_type
                )
            }
            Self::BinaryOPeratorNotDefinedOnType { op, lhs, rhs } => {
                format!(
                    "Binary operator '{:?}' not defined on types '{:?}' and '{:?}'",
                    op, rhs, lhs
                )
            }
            Self::EqualityNonMatchingTypes { lhs, rhs } => {
                format!(
                    "Equality operator must have matching types, found '{:?}' and '{:?}'",
                    rhs, rhs
                )
            }
        }
    }
}

pub struct Error {
    kind: ErrorKind,
    line: usize,
    column: usize,
}

impl Error {
    pub fn format(&self) -> String {
        let err_msg = self.kind.format();
        format!(
            "{} on line: {}, column: {}",
            err_msg, self.line, self.column
        )
    }
}

pub struct ErrorBag {
    errors: Vec<Error>,
}

impl ErrorBag {
    pub fn new() -> ErrorBag {
        ErrorBag {
            errors: Vec::<Error>::new(),
        }
    }

    pub fn add(&mut self, kind: ErrorKind, line: usize, col: usize) {
        let err = Error {
            kind: kind,
            line: line,
            column: col,
        };
        self.errors.push(err);
    }

    pub fn report(&self) {
        for err in &self.errors {
            let message = err.format();
            print!("ERR: {}", message);
        }
    }
}
