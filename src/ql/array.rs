use actix::Addr;
use reblessive::tree::{self, Stk};

use crate::{
    dbs::graph::Graph,
    doc::document::Cursor,
    err::Error,
    ql::{ident::Ident, number::Number, part::Part, value::Value},
};
use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

#[non_exhaustive]
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, Default)]
pub struct Array(pub Vec<Value>);

impl From<Vec<Value>> for Array {
    fn from(value: Vec<Value>) -> Self {
        Self(value)
    }
}

impl FromIterator<Value> for Array {
    fn from_iter<T: IntoIterator<Item = Value>>(iter: T) -> Self {
        Array(iter.into_iter().collect())
    }
}

impl Deref for Array {
    type Target = Vec<Value>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Array {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl IntoIterator for Array {
    type Item = Value;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Array {
    pub fn retrieve(&self, part: &Part) -> Result<Value, Error> {
        Ok(match part {
            Part::All => Value::Array(self.to_owned()),
            Part::Field(id) => Value::Array(self.get_field(id)),
            Part::Index(Number::Int(i)) => self.get(*i as usize).unwrap_or(&Value::None).to_owned(),
            _ => return Err(Error::InvalidIdiom),
        })
    }

    pub fn get(&self, i: usize) -> Option<&Value> {
        self.0.get(i)
    }

    fn get_field(&self, Ident(id): &Ident) -> Array {
        self.0
            .iter()
            .map(|val| {
                if let Value::Object(v) = val {
                    v.get(id).to_owned()
                } else {
                    Value::None
                }
            })
            .collect::<Vec<_>>()
            .into()
    }
}

impl Array {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(len: usize) -> Self {
        Array(Vec::with_capacity(len))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Display for Array {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut array_str = String::new();
        for e in self.0.iter() {
            array_str.push_str(&format!("{e},"));
        }
        array_str.pop();
        write!(f, "[{array_str}]")
    }
}

impl Array {
    pub async fn evaluate(
        &self,
        stk: &mut Stk,
        graph: &Addr<Graph>,
        cur: Option<&Cursor>,
    ) -> Result<Value, Error> {
        let mut array = Self::with_capacity(self.len());
        for val in self.iter() {
            let val = stk.run(|stk| val.evaluate(stk, graph, cur)).await?;
            array.push(val);
        }
        Ok(Value::Array(array))
    }
}
