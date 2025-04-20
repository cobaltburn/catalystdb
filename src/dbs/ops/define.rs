use crate::{
    dbs::{graph::Graph, table::Table},
    err::Error::{self, DefineError},
    resp::Response,
};
use actix::{Actor, Handler, Message};
use core::panic;

#[derive(Message, Debug)]
#[rtype(result = "Result<Response, Error>")]
#[non_exhaustive]
pub enum Define {
    Table(String),
    Index,
}

impl Handler<Define> for Graph {
    type Result = Result<Response, Error>;

    fn handle(&mut self, msg: Define, _ctx: &mut Self::Context) -> Self::Result {
        let Define::Table(table) = msg else {
            panic!("how did we get here");
        };

        if !self.tables.contains_key(&table) {
            self.tables.insert(table.clone(), Table::new(table).start());
        } else {
            return Err(DefineError(table));
        }
        Ok(Response::None)
    }
}
