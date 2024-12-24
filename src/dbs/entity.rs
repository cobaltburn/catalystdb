use crate::{
    dbs::ops::{delete::Delete, remove::Remove},
    err::Error,
    ql::{ident::Ident, record::Record, value::Value},
};
use actix::{Actor, Addr, AsyncContext, Context, Handler, Message};
use std::{collections::BTreeMap, sync::Arc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum Entity {
    Node {
        id: Record,
        fields: BTreeMap<Arc<str>, Value>,
        edges: BTreeMap<Record, Addr<Entity>>,
    },
    Edge {
        id: Record,
        fields: BTreeMap<Arc<str>, Value>,
        edges: BTreeMap<Record, Addr<Entity>>,
    },
}

impl Actor for Entity {
    type Context = Context<Self>;

    fn stopping(&mut self, _ctx: &mut Self::Context) -> actix::prelude::Running {
        match self {
            Entity::Node { edges, .. } => edges.iter().for_each(|(_, edge)| edge.do_send(Delete)),
            Entity::Edge { id, fields, edges } => {
                let in_rec: Record = fields
                    .get("in".into())
                    .expect("In field was not present")
                    .clone()
                    .try_into()
                    .unwrap();
                let out_rec: Record = fields
                    .get("out".into())
                    .expect("Out field was not present")
                    .clone()
                    .try_into()
                    .unwrap();
                let origin = edges.get(&in_rec).unwrap();
                let dest = edges.get(&out_rec).unwrap();
                origin.do_send(Remove::Edge(id.clone()));
                dest.do_send(Remove::Edge(id.clone()));
            }
        }
        actix::prelude::Running::Stop
    }

    fn start(self) -> Addr<Self>
    where
        Self: Actor<Context = Context<Self>>,
    {
        let is_edge = self.is_edge();
        let addr = Context::new().run(self);
        if is_edge {
            addr.do_send(Configure);
        }
        addr
    }
}

impl Entity {
    pub fn edges(&self) -> &BTreeMap<Record, Addr<Entity>> {
        match self {
            Entity::Node { edges, .. } => edges,
            Entity::Edge { edges, .. } => edges,
        }
    }

    pub fn id(&self) -> &Record {
        match self {
            Entity::Node { id, .. } => id,
            Entity::Edge { id, .. } => id,
        }
    }

    pub fn fields(&self) -> &BTreeMap<Arc<str>, Value> {
        match self {
            Entity::Node { fields, .. } => fields,
            Entity::Edge { fields, .. } => fields,
        }
    }

    pub fn get(&self, key: &Arc<str>) -> Option<&Value> {
        self.fields().get(key)
    }

    pub fn evaluate(&self, val: &Value) -> Result<Value, Error> {
        Ok(match val {
            v @ Value::Record(_) => v.to_owned(),
            Value::Idiom(v) => v.evaluate(&self.fields().clone().into())?,
            Value::Expression(v) => v.evaluate(&self.fields().clone().into())?,
            v => v.to_owned(),
        })
    }
}

// Node
impl Entity {
    pub fn is_node(&self) -> bool {
        match self {
            Entity::Node { .. } => true,
            _ => false,
        }
    }

    pub fn new_node(id: Record, fields: Vec<(Arc<str>, Value)>) -> Self {
        let mut fields: BTreeMap<_, _> = fields.into_iter().collect();
        fields.insert("id".into(), id.clone().into());
        Entity::Node {
            id,
            fields,
            edges: BTreeMap::new(),
        }
    }
}

// Edge
impl Entity {
    pub fn is_edge(&self) -> bool {
        match self {
            Entity::Edge { .. } => true,
            _ => false,
        }
    }

    pub fn new_edge(
        edge: String,
        dest_id: Record,
        org_id: Record,
        dest: Addr<Entity>,
        origin: Addr<Entity>,
        fields: Vec<(String, Value)>,
    ) -> Self {
        let mut fields: BTreeMap<Arc<str>, Value> =
            fields.into_iter().map(|(k, v)| (k.into(), v)).collect();
        fields.insert("in".into(), org_id.clone().into());
        fields.insert("out".into(), dest_id.clone().into());
        let edges = BTreeMap::from([(org_id, origin), (dest_id, dest)]);
        Entity::Edge {
            id: Record::new(edge, Uuid::new_v4().to_string()),
            fields,
            edges,
        }
    }

    pub fn bind_edges(&self, id: Record, node: Addr<Entity>) {
        if let Entity::Edge { fields, edges, .. } = self {
            let in_rec: Record = fields
                .get("in".into())
                .expect("In field was not present")
                .clone()
                .try_into()
                .unwrap();
            let out_rec: Record = fields
                .get("out".into())
                .expect("Out field was not present")
                .clone()
                .try_into()
                .unwrap();
            let origin = edges.get(&in_rec).unwrap();
            let dest = edges.get(&out_rec).unwrap();
            origin.do_send(Bind(id.clone(), node.clone()));
            dest.do_send(Bind(id, node));
        }
    }
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct Bind(pub Record, pub Addr<Entity>);

impl Handler<Bind> for Entity {
    type Result = ();

    fn handle(&mut self, Bind(id, address): Bind, _ctx: &mut Self::Context) -> Self::Result {
        if let Entity::Node { edges, .. } = self {
            edges.insert(id, address);
        };
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Configure;

impl Handler<Configure> for Entity {
    type Result = ();

    fn handle(&mut self, _msg: Configure, ctx: &mut Self::Context) -> Self::Result {
        self.bind_edges(self.id().clone(), ctx.address());
    }
}
