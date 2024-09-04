use crate::{
    graph::{edge::Edge, node::Node},
    ql::{record::Record, value::Value},
};
use actix::{dev::MessageResponse, Actor, Addr, Handler, Message};
use std::sync::Arc;

pub enum FieldIds {
    WildCard,
    Fields(Vec<Arc<str>>),
}

#[derive(Message)]
#[rtype(result = "GetResponses")]
pub struct Get(pub FieldIds);

impl Handler<Get> for Node {
    type Result = GetResponses;

    fn handle(&mut self, Get(fields): Get, _ctx: &mut Self::Context) -> Self::Result {
        match fields {
            FieldIds::WildCard => self.fields().into(),
            FieldIds::Fields(fields) => fields
                .into_iter()
                .map(|field| {
                    if &*field == "id" {
                        (field, self.id().into())
                    } else if let Some(value) = self.fields.get(&field) {
                        (field, value.clone())
                    } else {
                        (field, Value::Null)
                    }
                })
                .collect::<Vec<_>>()
                .into(),
        }
    }
}

#[derive(Debug)]
pub enum GetResponses {
    Fields(Vec<(Arc<str>, Value)>),
    Value(Value),
    None,
}

impl From<Vec<(Arc<str>, Value)>> for GetResponses {
    fn from(fields: Vec<(Arc<str>, Value)>) -> Self {
        GetResponses::Fields(fields)
    }
}

impl From<Value> for GetResponses {
    fn from(value: Value) -> Self {
        GetResponses::Value(value)
    }
}

impl From<Record> for GetResponses {
    fn from(value: Record) -> Self {
        GetResponses::Value(Value::Record(value))
    }
}

impl TryFrom<GetResponses> for Vec<(Arc<str>, Value)> {
    type Error = ();

    fn try_from(value: GetResponses) -> Result<Self, Self::Error> {
        if let GetResponses::Fields(fields) = value {
            return Ok(fields);
        }
        Err(())
    }
}

impl TryFrom<GetResponses> for Value {
    type Error = ();

    fn try_from(value: GetResponses) -> Result<Self, Self::Error> {
        if let GetResponses::Value(value) = value {
            return Ok(value);
        }
        Err(())
    }
}

impl<A, M> MessageResponse<A, M> for GetResponses
where
    A: Actor,
    M: Message<Result = GetResponses>,
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
