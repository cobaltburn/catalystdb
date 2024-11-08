use crate::{
    dbs::node::Node,
    err::Error,
    ql::{
        array::Array, expression::Expression, ident::Ident, idiom::Idiom, number::Number,
        object::Object, part::Part, record::Record, uuid::Uuid,
    },
};
use core::fmt;
use std::sync::Arc;

#[derive(Debug, Clone, Default, PartialEq, Hash, Eq, PartialOrd, Ord)]
pub struct Values(pub Vec<Value>);

#[derive(Debug, Clone, Default, PartialEq, Hash, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Value {
    #[default]
    None,
    Null,
    Record(Box<Record>),
    Uuid(Uuid),
    Number(Number),
    String(Arc<str>),
    Bool(bool),
    Array(Array),
    Object(Object),
    Idiom(Idiom),
    Expression(Box<Expression>),
}

impl<T> From<Option<T>> for Value
where
    Value: From<T>,
{
    fn from(value: Option<T>) -> Self {
        match value {
            Some(value) => Value::from(value),
            None => Value::Null,
        }
    }
}

impl From<Vec<Value>> for Value {
    fn from(value: Vec<Value>) -> Self {
        Value::Array(Array::from(value))
    }
}

impl From<Record> for Value {
    fn from(value: Record) -> Self {
        Value::Record(Box::new(value))
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::String(value.into())
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Value::String(value.into())
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Bool(value)
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Value::Number(value.into())
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Value::Number(value.into())
    }
}

impl Value {
    pub fn nested_retrieval(&self, part: Part) -> &Value {
        match part {
            Part::Field(Ident(ident)) => match self {
                Value::Array(_) => todo!(),
                Value::Object(v) => todo!(),
                Value::Record(_) => todo!(),
                _ => &Value::None,
            },
            Part::Index(Number::Int(i)) => match self {
                Value::Array(v) => &v[i as usize],
                _ => &Value::None,
            },
            _ => panic!(),
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(v) => *v,
            Value::Record(_) => true,
            Value::Uuid(_) => true,
            Value::Number(_) => true,
            Value::Object(v) => !v.is_empty(),
            Value::String(v) => !v.is_empty(),
            Value::Array(v) => !v.is_empty(),
            _ => false,
        }
    }

    pub fn evaluate(&self, node: &Node) -> Result<Value, Error> {
        match self {
            Value::Record(_) => todo!(),
            Value::Array(_) => todo!(),
            Value::Idiom(v) => todo!(),
            Value::Object(v) => todo!(),
            Value::Expression(v) => v.evaluate(node),
            v => Ok(v.to_owned()),
        }
    }

    pub fn retrieve(&self, part: &Part) -> Result<Value, Error> {
        Ok(match self {
            Value::Record(_) => todo!(),
            Value::Array(v) => v.retrieve(part)?,
            Value::Object(v) => v.retrieve(part)?,
            _ => Value::None,
        })
    }
}

impl Value {
    pub fn try_neg(self) -> Result<Self, Error> {
        Ok(match self {
            Value::Number(v) => Value::Number(v.try_neg()?),
            v => return Err(Error::InvalidNegative(v)),
        })
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::None => write!(f, "NONE"),
            Value::Null => write!(f, "NULL"),
            Value::Record(v) => write!(f, "{v}"),
            Value::Uuid(v) => write!(f, "{v}"),
            Value::Number(v) => write!(f, "{v}"),
            Value::String(v) => write!(f, "{v}"),
            Value::Bool(v) => write!(f, "{v}"),
            Value::Array(v) => write!(f, "{v}"),
            Value::Object(v) => write!(f, "{v}"),
            Value::Idiom(v) => write!(f, "{v}"),
            Value::Expression(v) => write!(f, "{v}"),
        }
    }
}
