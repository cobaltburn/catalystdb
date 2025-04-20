use crate::{
    err::Error,
    ql::{idiom::Idiom, part::Part, value::Value},
};
use std::{ops::Deref, vec};

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct Fields(pub Vec<Field>);

impl From<Field> for Fields {
    fn from(field: Field) -> Self {
        Fields(vec![field])
    }
}

impl From<Vec<Field>> for Fields {
    fn from(fields: Vec<Field>) -> Self {
        Fields(fields)
    }
}

impl IntoIterator for Fields {
    type Item = Field;
    type IntoIter = vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Deref for Fields {
    type Target = Vec<Field>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub enum Field {
    WildCard,
    Single { expr: Value, alias: Option<String> },
}

impl Field {
    pub fn new(expr: Value) -> Field {
        Field::Single { expr, alias: None }
    }

    pub fn new_alias(expr: Value, alias: String) -> Field {
        Field::Single {
            expr,
            alias: Some(alias),
        }
    }
}

impl From<Value> for Field {
    fn from(expr: Value) -> Self {
        Field::Single { expr, alias: None }
    }
}

impl TryFrom<Part> for Field {
    type Error = Error;

    fn try_from(part: Part) -> Result<Self, Self::Error> {
        Ok(match part {
            Part::All => Field::WildCard,
            Part::Value(v) => v.into(),
            Part::Field(_) => Field::Single {
                expr: Value::Idiom(Idiom(vec![part])),
                alias: None,
            },
            _ => {
                return Err(Error::FailedFromPart {
                    from: part,
                    into: "Field".to_string(),
                })
            }
        })
    }
}
