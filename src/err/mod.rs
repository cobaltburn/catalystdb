use crate::{
    ql::{operator::Operator, part::Part, path::Path, value::Value},
    resp::Response,
};
use serde::Serialize;
use thiserror::Error;

#[derive(PartialEq, Eq, Error, Debug)]
#[non_exhaustive]
pub enum Error {
    #[error("Can not create {table}:{id}")]
    CreateError { table: String, id: String },

    #[error("Failed to define: {0}")]
    DefineError(String),

    #[error("Invalid negative argument: {0}")]
    InvalidNegative(Value),

    #[error("Could not find field: {0}")]
    FieldNotFound(String),

    #[error("Invalid Idiom")]
    InvalidIdiom,

    #[error("Invalid Operator: {0}")]
    InvalidOperator(Operator),

    #[error("{0} is an invalid type for evaluating node")]
    InvalidEvaluationPart(String),

    #[error("Invalid addition types: {0}, {1}")]
    TryAdd(String, String),

    #[error("Invalid subtraction types: {0}, {1}")]
    TrySub(String, String),

    #[error("Invalid multiplication types: {0}, {1}")]
    TryMul(String, String),

    #[error("Invalid division types: {0}, {1}")]
    TryDiv(String, String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("{0} is an out of bounds")]
    OutOfBoundsIndex(usize),

    #[error("Failed to convert value {from} into a {into}")]
    FailedFromValue { from: Value, into: String },

    #[error("Failed to convert response {from} into a {into}")]
    FailedFromResponse { from: Response, into: String },

    #[error("Failed to convert part {from} into a {into}")]
    FailedFromPart { from: Part, into: String },

    #[error("Expected a {expected} found {result:?}")]
    IncorrectValueType { expected: String, result: Value },

    #[error("")]
    InvalidStatement(),

    #[error("traversed beyond edge limit no idea how you did this index: {0} path: {1:#?}")]
    EdgeIndexExceeded(usize, Vec<Path>),

    #[error("Table not found: {0}")]
    InvalidTable(String),
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}

impl From<serde_content::Error> for Error {
    fn from(err: serde_content::Error) -> Self {
        Self::Serialization(err.to_string())
    }
}
