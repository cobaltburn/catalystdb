use crate::ql::{record::Record, table::Table, value::Value};
use std::sync::Arc;

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct Document {
    pub id: Option<Arc<Record>>,
    pub table: Option<Table>,
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct Cursor {
    pub value: Value,
}

impl From<Value> for Cursor {
    fn from(value: Value) -> Self {
        Cursor { value }
    }
}
