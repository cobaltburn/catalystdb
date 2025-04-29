use crate::{
    dbs::entity::Entity,
    err::Error,
    ql::{record::Record, value::Value},
};
use actix::{dev::MessageResponse, Actor, Addr, Message};
use std::{collections::BTreeMap, fmt, sync::Arc};

#[derive(PartialEq, Eq, Debug)]
pub enum Response {
    Value(Value),
    Nodes(Vec<Addr<Entity>>),
    Node(Addr<Entity>),
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

    fn try_from(response: Response) -> Result<Self, Self::Error> {
        if let Response::Value(response) = response {
            return Ok(response);
        }
        Err(Error::FailedFromResponse {
            from: response,
            into: String::from("Value"),
        })
    }
}

impl TryFrom<Response> for Vec<Addr<Entity>> {
    type Error = Error;

    fn try_from(response: Response) -> Result<Self, Self::Error> {
        if let Response::Nodes(response) = response {
            return Ok(response);
        }
        Err(Error::FailedFromResponse {
            from: response,
            into: String::from("Vec<Addr<Entity>>"),
        })
    }
}

impl TryFrom<Response> for Addr<Entity> {
    type Error = Error;

    fn try_from(response: Response) -> Result<Self, Self::Error> {
        if let Response::Node(response) = response {
            return Ok(response);
        }
        Err(Error::FailedFromResponse {
            from: response,
            into: String::from("Addr<Entity>"),
        })
    }
}

impl From<Vec<Addr<Entity>>> for Response {
    fn from(records: Vec<Addr<Entity>>) -> Self {
        Response::Nodes(records)
    }
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Response::Value(v) => write!(f, "{v}"),
            Response::Nodes(v) => write!(f, "{v:#?}"),
            Response::Node(v) => write!(f, "{v:#?}"),
            Response::None => write!(f, "NONE"),
        }
    }
}
