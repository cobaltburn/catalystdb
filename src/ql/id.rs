use crate::{
    dbs::graph::Graph,
    doc::document::Cursor,
    err::Error,
    ql::{array::Array, object::Object, strand::Strand, uuid::Uuid, value::Value},
};
use actix::Addr;
use reblessive::tree::Stk;
use std::collections::BTreeMap;

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Id {
    Number(i64),
    String(String),
    Uuid(uuid::Uuid),
    Array(Array),
    Object(Object),
}

impl From<i64> for Id {
    fn from(v: i64) -> Self {
        Id::Number(v)
    }
}

impl From<i32> for Id {
    fn from(v: i32) -> Self {
        Id::Number(v as i64)
    }
}

impl From<u64> for Id {
    fn from(v: u64) -> Self {
        Id::Number(v as i64)
    }
}

impl From<u32> for Id {
    fn from(v: u32) -> Self {
        Id::Number(v as i64)
    }
}

impl From<String> for Id {
    fn from(v: String) -> Self {
        Id::String(v)
    }
}

impl From<&str> for Id {
    fn from(v: &str) -> Self {
        Id::String(v.to_owned())
    }
}

impl From<&String> for Id {
    fn from(v: &String) -> Self {
        Id::String(v.to_owned())
    }
}

impl From<Strand> for Id {
    fn from(v: Strand) -> Self {
        Id::String(v.as_string())
    }
}

impl From<Array> for Id {
    fn from(v: Array) -> Self {
        Id::Array(v)
    }
}

impl From<Object> for Id {
    fn from(v: Object) -> Self {
        Id::Object(v)
    }
}

impl From<Vec<Value>> for Id {
    fn from(v: Vec<Value>) -> Self {
        Id::Array(v.into())
    }
}

impl From<BTreeMap<String, Value>> for Id {
    fn from(v: BTreeMap<String, Value>) -> Self {
        Id::Object(v.into())
    }
}

impl Id {
    pub async fn evaluate(
        &self,
        stk: &mut Stk,
        graph: &Addr<Graph>,
        cur: Option<&Cursor>,
    ) -> Result<Id, Error> {
        match self {
            Id::Number(v) => Ok(Id::Number(*v)),
            Id::String(v) => Ok(Id::String(v.to_owned())),
            Id::Uuid(v) => Ok(Id::Uuid(*v)),
            Id::Array(v) => match v.evaluate(stk, graph, cur).await? {
                Value::Array(v) => Ok(Id::Array(v)),
                v => todo!(),
            },
            Id::Object(v) => match v.evaluate(stk, graph, cur).await? {
                Value::Object(v) => Ok(Id::Object(v)),
                v => todo!(),
            },
        }
    }
}
