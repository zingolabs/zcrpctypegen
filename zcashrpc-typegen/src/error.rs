pub type TypegenResult<T> = Result<T, TypegenError>;

#[derive(Debug, derive_more::From)]
pub enum TypegenError {
    Filesystem(FSError),
    InvalidJson(InvalidJsonError),
    InvalidAnnotation(InvalidAnnotationError),
}

#[derive(Debug)]
pub struct FSError {
    message: String,
    location: Box<std::path::Path>,
}

impl FSError {
    pub(crate) fn from_io_error(
        err: std::io::Error,
        location: Box<std::path::Path>,
    ) -> Self {
        Self {
            message: format!("{:?}", err.kind()),
            location,
        }
    }
}

#[derive(Debug)]
pub struct InvalidJsonError {
    err: serde_json::Error,
    input: String,
}

impl InvalidJsonError {
    pub fn from_serde_json_error(
        err: serde_json::Error,
        input: String,
    ) -> Self {
        Self { err, input }
    }
}

#[derive(Debug, derive_more::From)]
pub struct InvalidAnnotationError {
    pub kind: InvalidAnnotationKind,
    pub location: String,
}

#[derive(Debug)]
pub enum InvalidAnnotationKind {
    Null,
    Bool(bool),
    Number(rust_decimal::Decimal),
    InvalidString(String),
    EmptyArray,
}

impl From<serde_json::Value> for InvalidAnnotationKind {
    fn from(val: serde_json::Value) -> Self {
        match val {
            serde_json::Value::Null => Self::Null,
            serde_json::Value::Bool(b) => Self::Bool(b),
            serde_json::Value::Number(n) => Self::Number(
                serde_json::from_str(&n.to_string())
                    .expect(&format!("Invalid number: {}", n)),
            ),
            serde_json::Value::String(s) => Self::InvalidString(s),
            val => panic!("valid serde_json item {} converted to error", val),
        }
    }
}
