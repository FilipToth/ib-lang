pub enum ErrorKind {
    FailedParsing,
    NumberParsing,
}

impl ErrorKind {
    pub fn format(&self) -> String {
        let msg = match self {
            Self::FailedParsing => "Failed parsing",
            Self::NumberParsing => "Cannot parse number",
        };

        msg.to_string()
    }
}

pub struct Error {
    kind: ErrorKind,
    line: usize,
    column: usize
}

impl Error {
    pub fn format(&self) -> String {
        let err_msg = self.kind.format();
        format!("{} on line: {}, column: {}", err_msg, self.line, self.column)
    }
}

pub struct ErrorBag {
    errors: Vec<Error>
}

impl ErrorBag {
    pub fn new() -> ErrorBag {
        ErrorBag { errors: Vec::<Error>::new() }
    }

    pub fn add(&mut self, kind: ErrorKind, line: usize, col: usize) {
        let err = Error { kind: kind, line: line, column: col };
        self.errors.push(err);
    }

    pub fn report(&self) {
        for err in &self.errors {
            let message = err.format();
            print!("ERR: {}", message);
        }
    }
}
