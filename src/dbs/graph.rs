use crate::dbs::table::Table;
use actix::{Actor, Addr, Context, Handler, Message};
use std::collections::BTreeMap;

#[derive(Debug, Default)]
pub struct Graph {
    pub tables: BTreeMap<String, Addr<Table>>,
}

impl Actor for Graph {
    type Context = Context<Self>;
}

impl Graph {
    pub fn new() -> Self {
        Graph::default()
    }
}

#[derive(Message)]
#[rtype(result = "Option<Addr<Table>>")]
pub struct Retrieve(pub String);

impl Retrieve {
    pub fn new<S: Into<String>>(table: S) -> Self {
        Retrieve(table.into())
    }
}

impl Handler<Retrieve> for Graph {
    type Result = Option<Addr<Table>>;

    fn handle(&mut self, Retrieve(table): Retrieve, _ctx: &mut Self::Context) -> Self::Result {
        Some(self.tables.get(&table)?.clone())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::dbs::{graph, ops::define::Define};

    #[actix_rt::test]
    async fn test_graph_table_pass() {
        let addr = Graph::new().start();
        addr.send(Define::table("a")).await.unwrap().unwrap();
        let _ = addr.send(graph::Retrieve::new("a")).await.unwrap().unwrap();
    }

    #[actix_rt::test]
    async fn test_graph_table_fail() {
        let addr = Graph::new().start();
        addr.send(Define::table("a")).await.unwrap().unwrap();
        let result = addr.send(graph::Retrieve::new("b")).await.unwrap();
        assert!(result.is_none());
    }
}
