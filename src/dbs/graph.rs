use crate::{
    dbs::table::Table,
    ql::{table, value::Value},
};
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
pub struct Retrieve(pub Value);

impl Handler<Retrieve> for Graph {
    type Result = Option<Addr<Table>>;

    fn handle(&mut self, Retrieve(value): Retrieve, _ctx: &mut Self::Context) -> Self::Result {
        Some(match &value {
            Value::Record(record) => self.tables.get(&record.table.to_string())?.clone(),
            Value::Table(table::Table(table)) => self.tables.get(table)?.clone(),
            _ => return None,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::dbs::{graph, ops::define::Define};

    #[actix_rt::test]
    async fn test_graph_table_pass() {
        let addr = Graph::new().start();
        addr.send(Define::Table(String::from("a")))
            .await
            .unwrap()
            .unwrap();
        let _ = addr
            .send(graph::Retrieve("a".into()))
            .await
            .unwrap()
            .unwrap();
    }

    #[actix_rt::test]
    async fn test_graph_table_fail() {
        let addr = Graph::new().start();
        addr.send(Define::Table(String::from("a")))
            .await
            .unwrap()
            .unwrap();
        let result = addr.send(graph::Retrieve("b".into())).await.unwrap();
        assert!(result.is_none());
    }
}
