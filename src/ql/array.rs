use crate::{
    err::Error,
    ql::{ident::Ident, number::Number, part::Part, value::Value},
};
use std::fmt::Display;

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct Array(pub Vec<Value>);

impl From<Vec<Value>> for Array {
    fn from(value: Vec<Value>) -> Self {
        Self(value)
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
                    v.get(id).unwrap_or(&Value::None).to_owned()
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
