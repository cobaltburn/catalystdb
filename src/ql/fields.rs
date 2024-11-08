use crate::ql::value::Value;
use std::vec;

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct Fields(pub Vec<Field>);

impl Fields {
    pub fn new(fields: Vec<Field>) -> Self {
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
