use crate::{
    dbs::graph::Graph,
    doc::document::Cursor,
    err::Error,
    ql::{traits::Incoperate, value::Value},
};
use actix::Addr;
use core::fmt;
use reblessive::tree::Stk;
use std::{collections::BTreeMap, sync::Arc};

#[derive(Debug, Clone, Default, PartialEq, Hash, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub struct Condition(pub Value);

impl Condition {
    pub async fn evaluate(
        &self,
        stk: &mut Stk,
        graph: &Addr<Graph>,
        fields: &BTreeMap<Arc<str>, Value>,
        cur: Option<&Cursor>,
    ) -> Result<Value, Error> {
        let val = match &self.0 {
            Value::Idiom(v) if v.first().unwrap().is_field() => &v.incorperate(fields).into(),
            Value::Expression(v) => &v.as_ref().incorperate(fields).into(),
            v => v,
        };
        stk.run(|stk| val.evaluate(stk, graph, cur)).await
    }

    fn incorperate(&self, fields: &BTreeMap<Arc<str>, Value>) -> Condition {
        Condition(match &self.0 {
            Value::Idiom(v) => v.incorperate(fields).into(),
            Value::Expression(v) => v.as_ref().incorperate(fields).into(),
            _ => return self.to_owned(),
        })
    }
}

/* impl Deref for Condition {
    type Target = Value;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
} */

impl From<Value> for Condition {
    fn from(value: Value) -> Self {
        Condition(value)
    }
}

impl fmt::Display for Condition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WHERE {}", self.0)
    }
}
