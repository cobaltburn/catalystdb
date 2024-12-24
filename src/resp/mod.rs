use crate::{
    dbs::entity::Entity,
    err::Error,
    ql::{record::Record, value::Value},
};
use actix::{dev::MessageResponse, Actor, Addr, Message};
use std::{collections::BTreeMap, sync::Arc};

#[derive(PartialEq, Eq, Debug)]
pub enum Response {
    Value(Value),
    Table(Vec<Addr<Entity>>),
    None,
}

impl<A, M> MessageResponse<A, M> for Response
where
    A: Actor,
    M: Message<Result = Response>,
{
    fn handle(
        self,
        _ctx: &mut A::Context,
        tx: Option<actix::prelude::dev::OneshotSender<M::Result>>,
    ) {
        if let Some(tx) = tx {
            let _ = tx.send(self);
        }
    }
}

impl From<BTreeMap<Arc<str>, Value>> for Response {
    fn from(value: BTreeMap<Arc<str>, Value>) -> Self {
        Response::Value(value.into())
    }
}

impl From<Value> for Response {
    fn from(value: Value) -> Self {
        Response::Value(value)
    }
}

impl From<Record> for Response {
    fn from(value: Record) -> Self {
        Response::Value(Value::Record(Box::new(value)))
    }
}

impl TryFrom<Response> for Value {
    type Error = Error;

    fn try_from(value: Response) -> Result<Self, Self::Error> {
        if let Response::Value(value) = value {
            return Ok(value);
        }
        Err(Error::FailedInto(format!("{value:?}")))
    }
}

impl From<Vec<Addr<Entity>>> for Response {
    fn from(records: Vec<Addr<Entity>>) -> Self {
        Response::Table(records)
    }
}
impl TryInto<Vec<Addr<Entity>>> for Response {
    type Error = ();

    fn try_into(
        self,
    ) -> Result<Vec<Addr<Entity>>, <Response as TryInto<Vec<Addr<Entity>>>>::Error> {
        if let Response::Table(node) = self {
            return Ok(node);
        }
        Err(())
    }
}
