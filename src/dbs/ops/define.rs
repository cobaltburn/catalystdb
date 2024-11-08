use crate::dbs::{graph::Graph, table::Table};
use actix::{Actor, Handler, Message};
use core::panic;

#[derive(Message)]
#[rtype(result = "Result<(), String>")]
#[non_exhaustive]
pub enum Define {
    Table(String),
    Index,
}

impl Define {
    pub fn table<S: Into<String>>(table: S) -> Self {
        Define::Table(table.into())
    }
}

impl Handler<Define> for Graph {
    type Result = Result<(), String>;

    fn handle(&mut self, msg: Define, _ctx: &mut Self::Context) -> Self::Result {
        let Define::Table(table) = msg else {
            panic!("how did we get here");
        };

        if !self.tables.contains_key(&table) {
            self.tables.insert(table.clone(), Table::new(table).start());
        } else {
            return Err("Table already exists".into());
        }
        Ok(())
    }
}
