use crate::{
    dbs::node::Node,
    err::Error,
    ql::{ident::Ident, number::Number, value::Value},
};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Hash, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Part {
    All,
    Field(Ident),
    Index(Number),
    Value(Value),
}

impl Part {
    pub fn evaluate(&self, node: &Node) -> Result<Value, Error> {
        Ok(match self {
            Part::All => Value::Object(node.fields().into()),
            Part::Field(v) => node
                .get(v)
                .ok_or_else(|| Error::FieldNotFound(v.to_string()))?
                .clone(),
            v => return Err(Error::InvalidEvaluationPart(v.to_string())),
        })
    }
}

impl fmt::Display for Part {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Part::All => write!(f, "*"),
            Part::Field(v) => write!(f, "{v}"),
            Part::Index(v) => write!(f, "{v}"),
            Part::Value(v) => write!(f, "{v}"),
        }
    }
}
