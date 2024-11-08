use crate::{
    dbs::{node::Node, table::Table},
    err::Error,
    ql::{record::Record, value::Value},
};
use actix::{Actor, Handler, Message};
use std::sync::Arc;

#[derive(Message)]
#[rtype(result = "Result<(), Error>")]
pub struct Create(Value, Vec<(Arc<str>, Value)>);

impl Create {
    pub fn new<V: Into<Value>, W: Into<Arc<str>>>(value: V, fields: Vec<(W, Value)>) -> Self {
        let value = value.into();
        let fields = fields.into_iter().map(|(id, v)| (id.into(), v)).collect();
        Create(value, fields)
    }
}

impl Handler<Create> for Table {
    type Result = Result<(), Error>;

    fn handle(&mut self, Create(id, fields): Create, _ctx: &mut Self::Context) -> Self::Result {
        let table = self.name.clone();
        if !self.contains(&id) {
            let addr = Node::new(Record::new(table, id.clone()), fields).start();
            self.insert(id, addr);
            return Ok(());
        }
        Err(Error::CreateError {
            table: table.to_string(),
            id: String::from("todo"),
        })
    }
}
