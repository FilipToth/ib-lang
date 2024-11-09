use super::{binding::types::TypeKind, operator::Operator};

pub enum ErrorKind {
    // Parsing Errors
    ExpectedToken(String),
    ExpectedScope,
    UnknownOperator(String),
    ExpectedPrimaryExpressionGot(String),
    ExpectedPrimaryExpression,
    UnclosedParenthesisExpression,
    ExpectedArgument,
    ExpectedParameter,

    // Binding Errors
    FailedParsing,
    NumberParsing,
    AssignMismatchedTypes,
    ParamMismatchedTypes(String),
    CannotFindValue(String),
    CannotFindFunction(String),
    CannotDeclareFunction(String),
    MismatchedNumberOfArgs {
        id: String,
        expected: usize,
        found: usize,
    },
    MismatchedArgTypes {
        id: String,
        expected: TypeKind,
        found: TypeKind,
    },
    NotAllCodePathsReturn,
    ReturnTypeMismatch {
        found: TypeKind,
        expected: TypeKind,
    },
    ConditionMustBeBoolean(TypeKind),
    UndefinedType(String),
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
            Self::ExpectedToken(token) => format!("Expected token: {}", token),
            Self::ExpectedScope => "Expected scope".to_string(),
            Self::UnknownOperator(op) => format!("{} is not an operator", op),
            Self::ExpectedPrimaryExpressionGot(token) => format!("Expected a primary expression, got: {}", token),
            Self::ExpectedPrimaryExpression => "Expected a primary expression".to_string(),
            Self::UnclosedParenthesisExpression => "Unclosed parenthesis expression".to_string(),
            Self::ExpectedArgument => "Expected an argument".to_string(),
            Self::ExpectedParameter => "Expected a parameter".to_string(),

            Self::FailedParsing => "Failed parsing".to_string(),
            Self::NumberParsing => "Cannot parse number".to_string(),
            Self::AssignMismatchedTypes => "Mismatched types in assign expression".to_string(),
            Self::ParamMismatchedTypes(param) => format!("Cannot assign parameter '{}' because a value with a different type already exists in the current scope", param),
            Self::CannotFindValue(id) => format!("Cannot find value '{}' in the current scope", id),
            Self::CannotFindFunction(id) => format!("Cannot find function '{}' in the current scope", id),
            Self::CannotDeclareFunction(id) => format!("Cannot declare function '{}' because the scope already contains one with the same name", id),
            Self::MismatchedNumberOfArgs { id, expected, found } => format!("Expected {} arguments, found {} when calling function '{}'", expected, found, id),
            Self::MismatchedArgTypes { id, expected, found } => format!("Expected an argument of type {:?}, found {:?} when calling function {}", expected, found, id),
            Self::NotAllCodePathsReturn => "Not all code paths return".to_string(),
            Self::ReturnTypeMismatch { found, expected } => format!("Return type doesn't match function return type, found {:?}, expected {:?}", found, expected),
            Self::ConditionMustBeBoolean(cond_type) => {
                format!("Condition type must be boolean, found {:?}", cond_type)
            }
            Self::UndefinedType(type_ref) => {
                format!("Undefined type '{}' in the current scope", type_ref)
            }
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
                    lhs, rhs
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
    pub errors: Vec<Error>,
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
