use crate::{
    dbs::node::Node,
    ql::{
        fields::{Field, Fields},
        idiom::Idiom,
        part::Part,
        record::Record,
        value::Value,
    },
};
use actix::{dev::MessageResponse, Actor, Handler, Message};
use std::{collections::BTreeMap, sync::Arc};

#[derive(Message)]
#[rtype(result = "Response")]
pub struct Get {
    pub fields: Fields,
    pub filter: Option<Value>,
}

impl Get {
    pub fn new<V: Into<Fields>>(fields: V, filter: Option<Value>) -> Self {
        Get {
            fields: fields.into(),
            filter,
        }
    }
}

impl Handler<Get> for Node {
    type Result = Response;

    fn handle(&mut self, Get { fields, filter }: Get, _ctx: &mut Self::Context) -> Self::Result {
        if let Some(filter) = filter {
            let Value::Expression(expr) = filter else {
                return Response::None;
            };
        };
        let mut results = BTreeMap::new();
        for field in fields {
            match field {
                Field::WildCard => results.append(&mut self.fields()),
                Field::Single { expr, alias } => {
                    match expr {
                        Value::Idiom(Idiom(parts)) => {
                            let mut parts = parts.into_iter();
                            // TODO deal with if its not a field
                            let Some(Part::Field(ident)) = parts.next() else {
                                panic!();
                            };
                            let Some((key, value)) = self.get(&ident) else {
                                let key = alias.map_or(ident.0, |key| key.into());
                                results.insert(key, Value::None);
                                continue;
                            };
                            // TODO set it up to deal with objects and that good stuff
                            // for part in parts {}
                            let key = alias.map_or(key.into_owned(), |key| key.into());
                            results.insert(key, value.into_owned());
                        }
                        _v => todo!(),
                    }
                }
            };
        }
        Response::Fields(results)
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum Response {
    Fields(BTreeMap<Arc<str>, Value>),
    Value(Value),
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
    fn from(fields: BTreeMap<Arc<str>, Value>) -> Self {
        Response::Fields(fields)
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

impl TryFrom<Response> for BTreeMap<Arc<str>, Value> {
    // TODO: need to make an error type
    type Error = ();

    fn try_from(value: Response) -> Result<Self, Self::Error> {
        if let Response::Fields(fields) = value {
            return Ok(fields);
        }
        Err(())
    }
}

impl TryFrom<Response> for Value {
    type Error = ();

    fn try_from(value: Response) -> Result<Self, Self::Error> {
        if let Response::Value(value) = value {
            return Ok(value);
        }
        Err(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        dbs::{
            graph::{self, Graph},
            node::Node,
            ops::{create::Create, define::Define},
            table,
        },
        ql::{fields::Field, value::Value},
    };
    use actix::Addr;

    #[actix_rt::test]
    async fn get_test() {
        let graph = Graph::new().start();
        graph.send(Define::table("a")).await.unwrap().unwrap();
        let table = graph
            .send(graph::Retrieve::new("a"))
            .await
            .unwrap()
            .unwrap();

        let fields: Vec<(&str, _)> = vec![("1", Value::String("1".into()))];
        let _ = table
            .send(Create::new(1, fields.clone()))
            .await
            .unwrap()
            .unwrap();
        let _ = table
            .send(Create::new(2, fields.clone()))
            .await
            .unwrap()
            .unwrap();
        let _ = table
            .send(Create::new(3, fields.clone()))
            .await
            .unwrap()
            .unwrap();

        let nodes = table.send(table::Retrieve::Iterator).await.unwrap();
        let nodes: Vec<Addr<Node>> = nodes.try_into().unwrap();
        for node in nodes {
            let response = node
                .send(Get::new(vec![Field::WildCard], None))
                .await
                .unwrap();
            let fields: BTreeMap<_, _> = response.try_into().unwrap();
            let value = fields.get("1").unwrap();
            assert_eq!(*value, "1".into());
        }
    }
}
