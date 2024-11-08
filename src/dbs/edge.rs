use crate::{
    dbs::{
        node::{Bind, Node},
        ops::remove::Remove,
    },
    ql::{record::Record, value::Value},
};
use actix::{Actor, Addr, AsyncContext, Context, Handler, Message};
use std::{collections::BTreeMap, sync::Arc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Path<T: Actor> {
    pub id: Record,
    pub addr: Addr<T>,
}

impl<T: Actor> Path<T> {
    pub fn new(id: Record, addr: Addr<T>) -> Self {
        Path { id, addr }
    }

    pub fn id(&self) -> Record {
        self.id.clone()
    }

    pub fn address(&self) -> Addr<T> {
        self.addr.clone()
    }
}

#[derive(Debug, Clone)]
pub struct Edge {
    pub id: Record,
    pub fields: BTreeMap<Arc<str>, Value>,
    pub origin: Path<Node>,
    pub dest: Path<Node>,
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
            origin: Path::new(from, from_address),
            dest: Path::new(to, to_address),
        }
    }
    pub fn id(&self) -> Record {
        self.id.clone()
    }

    pub fn origin(&self) -> Addr<Node> {
        self.origin.addr.clone()
    }

    pub fn dest(&self) -> Addr<Node> {
        self.dest.addr.clone()
    }

    pub fn bind_origin(&self, msg: Bind) {
        self.origin.addr.do_send(msg)
    }

    pub fn bind_dest(&self, msg: Bind) {
        self.dest.addr.do_send(msg)
    }

    pub fn fields(&self) -> BTreeMap<Arc<str>, Value> {
        let mut fields = self.fields.clone();
        let origin = self.origin.id();
        let dest = self.dest.id();
        fields.insert("in".into(), origin.into());
        fields.insert("out".into(), dest.into());
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
        let addr_1 = self.origin.address();
        let addr_2 = self.dest.address();
        addr_1.do_send(Remove::Edge(self.id()));
        addr_2.do_send(Remove::Edge(self.id()));
        actix::prelude::Running::Stop
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Configure;

impl Handler<Configure> for Edge {
    type Result = ();

    fn handle(&mut self, _msg: Configure, ctx: &mut Self::Context) -> Self::Result {
        self.bind_origin(Bind(self.id(), ctx.address()));
        self.bind_dest(Bind(self.id(), ctx.address()));
    }
}
