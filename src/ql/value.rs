use crate::{
    dbs::graph::Graph,
    doc::document::Cursor,
    err::Error,
    ql::{
        array::Array, edge::Edge, expression::Expression, id::Id, ident::Ident, idiom::Idiom,
        number::Number, object::Object, part::Part, record::Record, strand::Strand, table::Table,
        uuid::Uuid,
    },
};
use actix::Addr;
use core::fmt;
use reblessive::tree::Stk;
use std::{
    collections::BTreeMap,
    ops::{Add, Deref, Div, Mul, Sub},
    sync::Arc,
};

#[derive(Debug, Clone, Default, PartialEq, Hash, Eq, PartialOrd, Ord)]
pub struct Values(pub Vec<Value>);

impl IntoIterator for Values {
    type Item = Value;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Deref for Values {
    type Target = Vec<Value>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Hash, Eq, PartialOrd, Ord)]
pub enum Value {
    #[default]
    None,
    Null,
    Record(Box<Record>),
    Uuid(Uuid),
    Number(Number),
    String(Strand),
    Bool(bool),
    Array(Array),
    Object(Object),
    Idiom(Idiom),
    Expression(Box<Expression>),
    Edge(Box<Edge>),
    Table(Table),
}

impl Value {
    pub fn is_none(&self) -> bool {
        matches!(self, Value::None)
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    pub fn is_record(&self) -> bool {
        matches!(self, Value::Record(_))
    }

    pub fn is_uuid(&self) -> bool {
        matches!(self, Value::Uuid(_))
    }

    pub fn is_number(&self) -> bool {
        matches!(self, Value::Number(_))
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }

    pub fn is_bool(&self) -> bool {
        matches!(self, Value::Bool(_))
    }

    pub fn is_array(&self) -> bool {
        matches!(self, Value::Array(_))
    }

    pub fn is_object(&self) -> bool {
        matches!(self, Value::Object(_))
    }

    pub fn is_idiom(&self) -> bool {
        matches!(self, Value::Idiom(_))
    }

    pub fn is_expression(&self) -> bool {
        matches!(self, Value::Expression(_))
    }

    pub fn is_edge(&self) -> bool {
        matches!(self, Value::Edge(_))
    }

    pub fn is_table(&self) -> bool {
        matches!(self, Value::Table(_))
    }
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

impl TryFrom<Value> for Vec<Value> {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::Array(array) = value {
            return Ok(array.0);
        }
        Err(Error::FailedFromValue {
            from: value,
            into: "Vec<Value>".to_string(),
        })
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

impl From<BTreeMap<Arc<str>, Value>> for Value {
    fn from(value: BTreeMap<Arc<str>, Value>) -> Self {
        Value::Object(Object(value))
    }
}

impl From<BTreeMap<String, Value>> for Value {
    fn from(v: BTreeMap<String, Value>) -> Self {
        let v = v.into_iter().map(|(s, v)| (s.into(), v)).collect();
        Value::Object(Object(v))
    }
}

impl From<Idiom> for Value {
    fn from(idiom: Idiom) -> Self {
        Value::Idiom(idiom)
    }
}

impl From<Vec<Part>> for Value {
    fn from(parts: Vec<Part>) -> Self {
        Value::Idiom(Idiom(parts))
    }
}

impl From<Part> for Value {
    fn from(part: Part) -> Self {
        Value::Idiom(Idiom(vec![part]))
    }
}

impl From<Ident> for Value {
    fn from(ident: Ident) -> Self {
        Value::Idiom(Idiom(vec![Part::Field(ident)]))
    }
}

impl From<Array> for Value {
    fn from(array: Array) -> Self {
        Value::Array(array)
    }
}

impl From<Table> for Value {
    fn from(table: Table) -> Self {
        Value::Table(table)
    }
}

impl From<Uuid> for Value {
    fn from(uuid: Uuid) -> Self {
        Value::Uuid(uuid)
    }
}

impl From<uuid::Uuid> for Value {
    fn from(uuid: uuid::Uuid) -> Self {
        Value::Uuid(uuid.into())
    }
}

impl From<Expression> for Value {
    fn from(expression: Expression) -> Self {
        Value::Expression(Box::new(expression))
    }
}

impl From<Id> for Value {
    fn from(id: Id) -> Self {
        match id {
            Id::Number(v) => Value::Number(v.into()),
            Id::String(v) => Value::String(v.into()),
            Id::Uuid(v) => Value::Uuid(v.into()),
            Id::Array(v) => Value::Array(v),
            Id::Object(v) => Value::Object(v),
        }
    }
}

impl TryFrom<Value> for Record {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::Record(record) = value {
            return Ok(*record);
        };
        Err(Error::FailedFromValue {
            from: value,
            into: String::from("Value"),
        })
    }
}

impl Value {
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

    pub async fn evaluate(
        &self,
        stk: &mut Stk,
        graph: &Addr<Graph>,
        cur: Option<&Cursor>,
    ) -> Result<Value, Error> {
        match self {
            Value::Record(v) => stk.run(|stk| v.evaluate(stk, graph, cur)).await,
            Value::Array(v) => stk.run(|stk| v.evaluate(stk, graph, cur)).await,
            Value::Object(v) => stk.run(|stk| v.evaluate(stk, graph, cur)).await,
            Value::Idiom(v) => stk.run(|stk| v.evaluate(stk, graph, cur)).await,
            Value::Expression(v) => stk.run(|stk| v.evaluate(stk, graph, cur)).await,
            _ => Ok(self.to_owned()),
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

    // pub fn filter(self, cond: &Condition) -> Result<Value, Error> {
    //     if self.evaluate(&cond.0)?.is_truthy() {
    //         return Ok(self);
    //     }
    //     Ok(Value::None)
    // }
}

impl Value {
    pub fn try_neg(self) -> Result<Self, Error> {
        Ok(match self {
            Value::Number(v) => Value::Number(v.try_neg()?),
            v => return Err(Error::InvalidNegative(v)),
        })
    }

    pub fn try_not(self) -> Result<Self, Error> {
        Ok(Value::Bool(!self.is_truthy()))
    }
}

impl Value {
    pub fn try_add(self, right: Value) -> Result<Value, Error> {
        Ok(match (self, right) {
            (Value::Number(left), Value::Number(right)) => Value::Number(left.add(right)),
            (Value::String(left), Value::String(right)) => Value::String(left.add(right)),
            (left, right) => return Err(Error::TryAdd(left.to_string(), right.to_string())),
        })
    }

    pub fn try_sub(self, right: Value) -> Result<Value, Error> {
        Ok(match (self, right) {
            (Value::Number(left), Value::Number(right)) => Value::Number(left.sub(right)),
            (left, right) => return Err(Error::TrySub(left.to_string(), right.to_string())),
        })
    }

    pub fn try_mul(self, right: Value) -> Result<Value, Error> {
        Ok(match (self, right) {
            (Value::Number(left), Value::Number(right)) => Value::Number(left.mul(right)),
            (left, right) => return Err(Error::TryMul(left.to_string(), right.to_string())),
        })
    }

    pub fn try_div(self, right: Value) -> Result<Value, Error> {
        Ok(match (self, right) {
            (Value::Number(left), Value::Number(right)) => Value::Number(left.div(right)),
            (left, right) => return Err(Error::TryDiv(left.to_string(), right.to_string())),
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
            Value::Table(Table(v)) => write!(f, "{v}"),
            Value::Edge(v) => write!(f, "{v}"),
        }
    }
}
