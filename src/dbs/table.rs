use crate::{
    dbs::node::Node,
    ql::{record::Record, value::Value},
};
use actix::{dev::MessageResponse, Actor, Addr, Context, Handler, Message};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{collections::HashMap, sync::Arc};

#[derive(Debug)]
pub struct Row(pub Record, pub Addr<Node>);

impl Row {
    pub fn addr(&self) -> Addr<Node> {
        let Row(_, addr) = self;
        addr.clone()
    }
}

#[derive(Debug)]
pub struct Table {
    pub name: Arc<str>,
    pub nodes: HashMap<Value, Addr<Node>>,
}

impl Actor for Table {
    type Context = Context<Self>;
}

impl Table {
    pub fn new<S: Into<Arc<str>>>(name: S) -> Self {
        Table {
            name: name.into(),
            nodes: HashMap::new(),
        }
    }

    pub fn insert(&mut self, value: Value, node: Addr<Node>) {
        self.nodes.insert(value, node);
    }

    pub fn contains(&self, value: &Value) -> bool {
        self.nodes.contains_key(value)
    }

    pub fn get(&self, record: Value) -> Option<Addr<Node>> {
        Some(self.nodes.get(&record)?.clone())
    }
}

#[derive(Message)]
#[rtype(result = "Response")]
pub enum Retrieve {
    Iterator,
    Record(Record),
}

impl Handler<Retrieve> for Table {
    type Result = Response;

    fn handle(&mut self, msg: Retrieve, _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            Retrieve::Iterator => {
                Response::Iterator(self.nodes.par_iter().map(|node| node.1.clone()).collect())
            }
            Retrieve::Record(Record { table: _, id }) => {
                self.get(id).map_or(Response::None, Into::into)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Response {
    Iterator(Vec<Addr<Node>>),
    Record(Addr<Node>),
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

impl From<Addr<Node>> for Response {
    fn from(record: Addr<Node>) -> Self {
        Response::Record(record)
    }
}

impl From<Vec<Addr<Node>>> for Response {
    fn from(records: Vec<Addr<Node>>) -> Self {
        Response::Iterator(records)
    }
}

impl TryInto<Addr<Node>> for Response {
    type Error = ();

    fn try_into(self) -> Result<Addr<Node>, Self::Error> {
        if let Response::Record(node) = self {
            return Ok(node);
        }
        Err(())
    }
}

impl TryInto<Vec<Addr<Node>>> for Response {
    type Error = ();

    fn try_into(self) -> Result<Vec<Addr<Node>>, Self::Error> {
        if let Response::Iterator(node) = self {
            return Ok(node);
        }
        Err(())
    }
}
