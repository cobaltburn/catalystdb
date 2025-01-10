use crate::ql::value::Value;
use std::{ops::Deref, vec};

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct Fields(pub Vec<Field>);

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

impl From<Vec<Field>> for Fields {
    fn from(fields: Vec<Field>) -> Self {
        Fields(fields)
    }
}
