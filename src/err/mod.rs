use crate::ql::{operator::Operator, value::Value};
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    #[error("Can not create {table}:{id}")]
    CreateError { table: String, id: Value },

    #[error("Invalid negative argument: {0}")]
    InvalidNegative(Value),

    #[error("Could not find field: {0}")]
    FieldNotFound(String),

    #[error("Invalid Idiom")]
    InvalidIdiom,

    #[error("Invalid Operator: {0}")]
    InvalidOperator(Operator),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("{0} is an out of bounds")]
    OutOfBoundsIndex(usize),
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
