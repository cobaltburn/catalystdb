use crate::{
    dbs::{entity::Entity, table::Table},
    err::Error::{self, CreateError},
    ql::{id::Id, record::Record, value::Value},
    resp::Response,
};
use actix::{Actor, Handler, Message};
use std::sync::Arc;

#[derive(Message)]
#[rtype(result = "Result<Response, Error>")]
pub struct Insert(pub Id, pub Vec<(Arc<str>, Value)>);

impl Insert {
    pub fn new<T: Into<Arc<str>>>(id: Id, fields: Vec<(T, Value)>) -> Insert {
        let fields = fields.into_iter().map(|(e, v)| (e.into(), v)).collect();
        Insert(id, fields)
    }
}

impl Handler<Insert> for Table {
    type Result = Result<Response, Error>;

    fn handle(&mut self, Insert(id, fields): Insert, _ctx: &mut Self::Context) -> Self::Result {
        let table = self.name.clone();
        if !self.contains(&id.clone().into()) {
            let node = Entity::new_node(Record::new(table, id.clone()), fields);
            let fields = node.fields().clone();
            let addr = node.start();
            self.insert(id.into(), addr);
            return Ok(Response::Value(fields.into()));
        }

        Err(CreateError {
            table: table.to_string(),
            id: id.to_string(),
        })
    }
}
