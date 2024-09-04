use crate::graph::{node::Bind, node::Node, remove::Remove};
use crate::ql::{record::Record, value::Value};
use actix::{Actor, Addr, AsyncContext, Context, Handler, Message};
use std::{collections::BTreeMap, sync::Arc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Path<T: Actor> {
    pub id: Record,
    pub address: Addr<T>,
}

impl<T: Actor> Path<T> {
    pub fn new(id: Record, address: Addr<T>) -> Self {
        Path { id, address }
    }

    pub fn id(&self) -> Record {
        self.id.clone()
    }

    pub fn address(&self) -> Addr<T> {
        self.address.clone()
    }
}

#[derive(Debug, Clone)]
pub struct Edge {
    pub id: Record,
    pub fields: BTreeMap<Arc<str>, Value>,
    pub node_1: Path<Node>,
    pub node_2: Path<Node>,
}

impl Edge {
    pub fn new<S: Into<Arc<str>>>(
        edge: S,
        to: Record,
        from: Record,
        to_address: Addr<Node>,
        from_address: Addr<Node>,
        fields: Vec<(String, Value)>,
    ) -> Self {
        Edge {
            id: Record::new(edge, Uuid::new_v4().to_string()),
            fields: fields.into_iter().map(|(k, v)| (k.into(), v)).collect(),
            node_1: Path::new(from, from_address),
            node_2: Path::new(to, to_address),
        }
    }
    pub fn id(&self) -> Record {
        self.id.clone()
    }

    pub fn fields(&self) -> Vec<(Arc<str>, Value)> {
        let mut fields = self
            .fields
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<Vec<_>>();
        let node_1 = self.node_1.id();
        let node_2 = self.node_2.id();
        fields.append(&mut vec![
            ("in".into(), Value::Record(node_1)),
            ("out".into(), Value::Record(node_2)),
        ]);
        fields
    }
}

impl Actor for Edge {
    type Context = Context<Self>;

    fn start(self) -> Addr<Self>
    where
        Self: Actor<Context = Context<Self>>,
    {
        let addr = Context::new().run(self);
        addr.do_send(Configure);
        addr
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> actix::prelude::Running {
        let addr_1 = self.node_1.address();
        let addr_2 = self.node_2.address();
        addr_1.do_send(Remove::<&str>::Edge(self.id()));
        addr_2.do_send(Remove::<&str>::Edge(self.id()));
        actix::prelude::Running::Stop
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Configure;

impl Handler<Configure> for Edge {
    type Result = ();

    fn handle(&mut self, _msg: Configure, ctx: &mut Self::Context) -> Self::Result {
        self.node_1
            .address()
            .do_send(Bind(self.id(), ctx.address()));
        self.node_2
            .address()
            .do_send(Bind(self.id(), ctx.address()));
    }
}
