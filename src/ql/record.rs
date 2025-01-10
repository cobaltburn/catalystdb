use super::uuid::Uuid;
use crate::ql::value::Value;
use std::{fmt, sync::Arc};

// #[derive(Debug, Hash, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct Record {
    pub table: Arc<str>,
    pub id: Value,
}

impl Record {
    pub fn new<T: Into<Arc<str>>, W: Into<Value>>(table: T, id: W) -> Self {
        Record {
            table: table.into(),
            id: id.into(),
        }
    }

    pub fn generate<T: Into<Arc<str>>>(table: T) -> Self {
        Record {
            table: table.into(),
            id: Uuid::new().into(),
        }
    }
}

impl fmt::Display for Record {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.table, self.id)
    }
}

impl<T: Into<Arc<str>>, W: Into<Value>> From<(T, W)> for Record {
    fn from((table, id): (T, W)) -> Self {
        Record::new(table, id)
    }
}
