use crate::{
    dbs::{entity::Entity, table::Table},
    err::Error,
    ql::{id::Id, record::Record, value::Value},
    resp::Response,
};
use actix::{Actor, Handler, Message};
use std::sync::Arc;

#[derive(Message)]
#[rtype(result = "Result<Response, Error>")]
pub struct Create(Id, Vec<(Arc<str>, Value)>);

impl Create {
    pub fn new<T: Into<Arc<str>>>(id: Id, fields: Vec<(T, Value)>) -> Create {
        let fields = fields.into_iter().map(|(e, v)| (e.into(), v)).collect();
        Create(id, fields)
    }
}

impl Handler<Create> for Table {
    type Result = Result<Response, Error>;

    fn handle(&mut self, Create(id, fields): Create, _ctx: &mut Self::Context) -> Self::Result {
        let table = self.name.clone();
        if !self.contains(&id.clone().into()) {
            let node = Entity::new_node(Record::new(table, id.clone()), fields);
            let fields = node.fields().clone();
            let addr = node.start();
            self.insert(id.into(), addr);
            return Ok(Response::Value(fields.into()));
        }
        Err(Error::CreateError {
            table: table.to_string(),
            id: String::from(id.to_string()),
        })
    }
}
