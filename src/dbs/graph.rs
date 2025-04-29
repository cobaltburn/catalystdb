use crate::dbs::table::Table;
use actix::{Actor, Addr, Context};
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

#[cfg(test)]
mod test {
    use core::panic;

    use super::*;
    use crate::{
        dbs::{
            graph,
            ops::{define::Define, retrieve::Retrieve},
        },
        resp::Response,
    };

    #[actix_rt::test]
    async fn test_graph_table_pass() {
        let addr = Graph::new().start();
        addr.send(Define::Table(String::from("a")))
            .await
            .unwrap()
            .unwrap();
        let _ = addr.send(Retrieve::Table("a".to_string())).await.unwrap();
    }

    #[actix_rt::test]
    async fn test_graph_table_fail() {
        let addr = Graph::new().start();
        addr.send(Define::Table(String::from("a")))
            .await
            .unwrap()
            .unwrap();
        let response: Response = addr.send(Retrieve::Table("b".into())).await.unwrap();
        let Response::Nodes(table) = response else {
            panic!("table was not returned");
        };
        assert_eq!(table.len(), 0);
    }
}
