use crate::{
    dbs::ops::{delete::Delete, remove::Remove},
    err::Error,
    ql::{direction::Direction, record::Record, value::Value},
};
use actix::{Actor, Addr, AsyncContext, Context, Handler, Message};
use std::{collections::BTreeMap, sync::Arc};
use uuid::Uuid;

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum Entity {
    Node {
        id: Record,
        fields: BTreeMap<Arc<str>, Value>,
        edges: BTreeMap<Record, Path>,
    },
    Edge {
        id: Record,
        fields: BTreeMap<Arc<str>, Value>,
        edges: BTreeMap<Record, Path>,
    },
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Path {
    In(Addr<Entity>),
    Out(Addr<Entity>),
}

impl Path {
    pub fn edge(&self) -> &Addr<Entity> {
        match self {
            Path::In(addr) => addr,
            Path::Out(addr) => addr,
        }
    }

    pub fn is_in(&self) -> bool {
        matches!(self, Path::In(_))
    }

    pub fn is_out(&self) -> bool {
        matches!(self, Path::Out(_))
    }

    pub fn valid_path(&self, dir: &Direction, entity: &Entity) -> Option<&Addr<Entity>> {
        Some(match dir {
            Direction::In if entity.is_edge() && self.is_out() => self.edge(),
            Direction::Out if entity.is_edge() && self.is_in() => self.edge(),
            Direction::In if entity.is_node() && self.is_in() => self.edge(),
            Direction::Out if entity.is_node() && self.is_out() => self.edge(),
            Direction::Both => self.edge(),
            _ => return None,
        })
    }
}

impl Actor for Entity {
    type Context = Context<Self>;

    fn stopping(&mut self, _ctx: &mut Self::Context) -> actix::prelude::Running {
        match self {
            Entity::Node { edges, .. } => edges
                .iter()
                .for_each(|(_, path)| path.edge().do_send(Delete)),
            Entity::Edge { id, fields, edges } => {
                let in_key: Arc<str> = "in".into();
                let out_key: Arc<str> = "out".into();

                let in_rec: Record = fields
                    .get(&in_key)
                    .expect("In field was not present")
                    .clone()
                    .try_into()
                    .unwrap();
                let out_rec: Record = fields
                    .get(&out_key)
                    .expect("Out field was not present")
                    .clone()
                    .try_into()
                    .unwrap();
                let origin = edges.get(&in_rec).unwrap().edge();
                let dest = edges.get(&out_rec).unwrap().edge();
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
    pub fn edges(&self) -> &BTreeMap<Record, Path> {
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
        todo!()
        /* Ok(match val {
            Value::Idiom(v) => v.evaluate(&self.fields().clone().into())?,
            Value::Expression(v) => v.evaluate(&self.fields().clone().into())?,
            _ => val.to_owned(),
        }) */
    }
}

// Node
impl Entity {
    pub fn is_node(&self) -> bool {
        matches!(self, Entity::Node { .. })
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
        matches!(self, Entity::Edge { .. })
    }

    pub fn new_edge(
        edge: String,
        dest_id: Record,
        org_id: Record,
        destination: Addr<Entity>,
        origin: Addr<Entity>,
        fields: Vec<(String, Value)>,
    ) -> Self {
        let mut fields: BTreeMap<Arc<str>, Value> =
            fields.into_iter().map(|(k, v)| (k.into(), v)).collect();

        fields.insert("in".into(), org_id.clone().into());
        fields.insert("out".into(), dest_id.clone().into());

        let edges = BTreeMap::from([
            (org_id, Path::In(origin)),
            (dest_id, Path::Out(destination)),
        ]);

        Entity::Edge {
            id: Record::new(edge, Uuid::new_v4().to_string()),
            fields,
            edges,
        }
    }

    pub fn bind_edges(&self, id: Record, node: Addr<Entity>) {
        if let Entity::Edge { fields, edges, .. } = self {
            let in_key: Arc<str> = "in".into();
            let out_key: Arc<str> = "out".into();

            let in_rec: Record = fields
                .get(&in_key)
                .expect("In field was not present")
                .clone()
                .try_into()
                .unwrap();
            let out_rec: Record = fields
                .get(&out_key)
                .expect("Out field was not present")
                .clone()
                .try_into()
                .unwrap();
            let origin = edges.get(&in_rec).unwrap().edge();
            let dest = edges.get(&out_rec).unwrap().edge();
            origin.do_send(Bind(id.clone(), Path::In(node.clone())));
            dest.do_send(Bind(id, Path::Out(node)));
        }
    }
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct Bind(pub Record, pub Path);

impl Handler<Bind> for Entity {
    type Result = ();

    fn handle(&mut self, Bind(id, address): Bind, _ctx: &mut Self::Context) -> Self::Result {
        let edges = match self {
            Entity::Node { edges, .. } => edges,
            Entity::Edge { edges, .. } => edges,
        };
        edges.insert(id, address);
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
