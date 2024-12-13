use crate::{
    dbs::node::Node,
    dbs::ops::response::Response,
    ql::{record::Record, value::Value},
};
use actix::{Actor, Addr, Context, Handler, Message};
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
                Response::Table(self.nodes.par_iter().map(|node| node.1.clone()).collect())
            }
            Retrieve::Record(Record { table: _, id }) => self
                .get(id)
                .map_or(Response::None, |addr| Response::Table(vec![addr])),
        }
    }
}
