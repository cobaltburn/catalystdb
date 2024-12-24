use crate::ql::{operator::Operator, value::Value};
use serde::Serialize;
use thiserror::Error;

#[derive(PartialEq, Eq, Error, Debug, Default)]
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

    #[error("Failed to convert {0}")]
    FailedInto(String),

    #[error("for returning a None value within the error")]
    #[default]
    None,
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
