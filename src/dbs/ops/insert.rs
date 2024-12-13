use crate::{
    dbs::{node::Node, table::Table},
    ql::{record::Record, value::Value},
};
use actix::{Actor, Handler, Message};
use std::sync::Arc;

#[derive(Message)]
#[rtype(result = "Result<(), String>")]
pub struct Insert(Value, Vec<(Arc<str>, Value)>);

impl Insert {
    pub fn new<V: Into<Value>, W: Into<Arc<str>>>(value: V, fields: Vec<(W, Value)>) -> Self {
        let value = value.into();
        let fields = fields.into_iter().map(|(id, v)| (id.into(), v)).collect();
        Insert(value, fields)
    }
}

impl Handler<Insert> for Table {
    type Result = Result<(), String>;

    fn handle(&mut self, Insert(id, fields): Insert, _ctx: &mut Self::Context) -> Self::Result {
        let table = self.name.clone();
        if !self.contains(&id) {
            let addr = Node::new(Record::new(table, id.clone()), fields).start();
            self.insert(id, addr);
            return Ok(());
        }
        Err(String::from("Record already exists"))
    }
}
