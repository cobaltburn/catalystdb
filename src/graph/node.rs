use crate::{
    graph::{delete::Delete, edge::Edge},
    ql::{record::Record, value::Value},
};
use actix::{Actor, Addr, Context, Handler, Message};
use std::{collections::BTreeMap, sync::Arc};

#[derive(Debug, Clone)]
pub struct Node {
    pub id: Record,
    pub fields: BTreeMap<Arc<str>, Value>,
    pub edges: BTreeMap<Record, Addr<Edge>>,
}

impl<R: Into<Record>, W: Into<Arc<str>>> From<(R, Vec<(W, Value)>)> for Node {
    fn from((id, fields): (R, Vec<(W, Value)>)) -> Self {
        Node {
            id: id.into(),
            fields: fields.into_iter().map(|(k, v)| (k.into(), v)).collect(),
            edges: BTreeMap::new(),
        }
    }
}

impl Node {
    pub fn id(&self) -> Record {
        self.id.clone()
    }

    pub fn new(id: Record, fields: Vec<(Arc<str>, Value)>) -> Self {
        Node {
            id,
            fields: fields.into_iter().collect(),
            edges: BTreeMap::new(),
        }
    }

    pub fn spawn(id: Record, fields: Vec<(Arc<str>, Value)>) -> Addr<Self> {
        Node {
            id,
            fields: fields.into_iter().collect(),
            edges: BTreeMap::new(),
        }
        .start()
    }
    pub fn fields(&self) -> Vec<(Arc<str>, Value)> {
        let mut fields = self
            .fields
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<Vec<_>>();
        fields.push((Arc::from("id"), self.id().into()));
        fields
    }
}

impl Actor for Node {
    type Context = Context<Self>;

    fn stopping(&mut self, _ctx: &mut Self::Context) -> actix::prelude::Running {
        let edges = self.edges.iter().collect::<Vec<_>>();
        for (_, addr) in edges {
            addr.do_send(Delete);
        }
        actix::prelude::Running::Stop
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Bind(pub Record, pub Addr<Edge>);

impl Handler<Bind> for Node {
    type Result = ();

    fn handle(&mut self, Bind(id, address): Bind, _ctx: &mut Self::Context) -> Self::Result {
        self.edges.insert(id, address);
    }
}
