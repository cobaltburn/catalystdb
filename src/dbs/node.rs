use crate::{
    dbs::{edge::Edge, ops::delete::Delete},
    err::Error,
    ql::{ident::Ident, idiom::Idiom, part::Part, record::Record, value::Value},
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
        let mut fields: BTreeMap<_, _> = fields.into_iter().collect();
        fields.insert("id".into(), id.clone().into());
        Node {
            id,
            fields,
            edges: BTreeMap::new(),
        }
    }

    pub fn fields(&self) -> BTreeMap<Arc<str>, Value> {
        self.fields.clone()
    }

    pub fn get(&self, Ident(id): &Ident) -> Option<&Value> {
        self.fields.get(id)
    }

    pub fn evaluate(&self, val: &Value) -> Result<Value, Error> {
        Ok(match val {
            v @ Value::Record(_) => v.to_owned(),
            Value::Idiom(v) => self.nested_field(&v)?,
            Value::Expression(v) => v.evaluate(self)?,
            v => v.to_owned(),
        })
    }

    pub fn nested_field(&self, Idiom(parts): &Idiom) -> Result<Value, Error> {
        let mut parts = parts.into_iter();
        let Some(part) = parts.next() else {
            return Ok(Value::None);
        };
        let field = match part {
            Part::Field(id) => self.get(&id),
            _ => return Err(Error::InvalidIdiom),
        };
        let Some(field) = field else {
            return Ok(Value::None);
        };
        let mut field = field.to_owned();

        for part in parts {
            field = field.retrieve(part)?;
        }

        todo!()
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
