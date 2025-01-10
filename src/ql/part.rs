use crate::{
    err::Error,
    ql::{edge::Edge, ident::Ident, number::Number, step::Step, value::Value},
};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Hash, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Part {
    All,
    Field(Ident),
    Index(Number),
    Value(Value),
    Step(Step),
    Edge(Edge),
}

impl Part {
    pub fn evaluate(&self, value: &Value) -> Result<Value, Error> {
        Ok(match self {
            Part::All => value.clone(),
            Part::Field(Ident(v)) => value.get(v).to_owned(),
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
            Part::Edge(v) => write!(f, "{v}"),
            Part::Step(v) => write!(f, "{v}"),
        }
    }
}
