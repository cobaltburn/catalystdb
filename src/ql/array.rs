use crate::{
    err::Error,
    ql::{ident::Ident, number::Number, part::Part, value::Value},
};
use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub struct Array(pub Vec<Value>);

impl From<Value> for Array {
    fn from(value: Value) -> Self {
        Array(vec![value])
    }
}

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

    pub fn len(&self) -> usize {
        self.0.len()
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
