use super::uuid::Uuid;
use crate::{
    dbs::graph::Graph,
    doc::document::Cursor,
    err::Error,
    ql::{id::Id, value::Value},
};
use actix::Addr;
use reblessive::tree::Stk;
use std::{fmt, sync::Arc};

// #[derive(Debug, Hash, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct Record {
    pub table: Arc<str>,
    pub id: Id,
}

impl Record {
    pub async fn evaluate(
        &self,
        stk: &mut Stk,
        graph: &Addr<Graph>,
        cur: Option<&Cursor>,
    ) -> Result<Value, Error> {
        Ok(Value::Record(Box::new(Record {
            table: self.table.clone(),
            id: stk.run(|stk| self.id.evaluate(stk, graph, cur)).await?,
        })))
    }
}

impl Record {
    pub fn new<T: Into<Arc<str>>, W: Into<Id>>(table: T, id: W) -> Self {
        Record {
            table: table.into(),
            id: id.into(),
        }
    }

    pub fn generate<T: Into<Arc<str>>>(table: T) -> Self {
        Record {
            table: table.into(),
            id: Id::default(),
        }
    }
}

impl fmt::Display for Record {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.table, self.id)
    }
}

impl<T: Into<Arc<str>>, W: Into<Id>> From<(T, W)> for Record {
    fn from((table, id): (T, W)) -> Self {
        Record::new(table, id)
    }
}
