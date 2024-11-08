use crate::{
    dbs::{
        graph::{self, Graph},
        node::Node,
        ops::get::{self, Get},
        table,
    },
    ql::{array::Array, fields::Fields, idiom::Idiom, value::Value},
};
use actix::Addr;
use anyhow::anyhow;
use std::ops::Deref;

#[non_exhaustive]
pub struct Select {
    pub fields: Fields,
    pub from: Value,
}

impl Select {
    pub fn new(fields: Fields, from: Value) -> Self {
        Select { fields, from }
    }

    pub async fn execute(self, graph: Addr<Graph>) -> anyhow::Result<Vec<get::Response>> {
        match &self.from {
            Value::Record(record) => {
                let msg = graph::Retrieve::new(record.table.deref());
                let table = graph.send(msg).await?.ok_or(anyhow!("didn't find table"))?;

                let msg = table::Retrieve::Record(*record.clone());
                let response = table.send(msg).await?;

                let node: Addr<Node> = response.try_into().unwrap();
                let response = node.send(Get::new(self.fields, None)).await?;
                Ok(vec![response])
            }
            Value::Idiom(Idiom(_parts)) => {
                /* let msg = graph::Retrieve::new(table.as_ref());
                let table = graph.send(msg).await?.ok_or(anyhow!("didn't find table"))?;

                let msg = table::Retrieve::Iterator;
                let response = table.send(msg).await?;

                let nodes: Vec<Addr<Node>> = response.try_into().unwrap();
                let mut results = Vec::new();
                for node in nodes {
                    let response = node.send(Get::new(self.fields.clone(), None)).await?;
                    results.push(response);
                }
                Ok(results) */
                todo!()
            }
            Value::Array(Array(_)) => Err(anyhow!("")),
            _ => Err(anyhow!("")),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        dbs::ops::{define::Define, insert::Insert},
        ql::{self, fields::Field, record::Record, value::Value},
    };
    use actix::Actor;
    use get::Response;
    use std::collections::BTreeMap;

    /* #[actix_rt::test]
    async fn test_select_table() {
        let select = Select::new(
            ql::fields::Fields::new(vec![Field::WildCard]),
            Value::Idiom(Ident::new("a")),
        );

        let graph = Graph::new().start();
        graph.send(Define::table("a")).await.unwrap();

        let msg = graph::Retrieve::new("a");
        let table = graph.send(msg).await.unwrap().unwrap();
        let _ = table.send(Insert::new(1, vec![("a", 1.into())])).await;
        let _ = table.send(Insert::new(2, vec![("a", 2.into())])).await;
        let _ = table.send(Insert::new(3, vec![("a", 3.into())])).await;

        let response = select.execute(graph).await.unwrap();
        let correct = vec![
            Response::Fields(BTreeMap::from([
                ("a".into(), 1.into()),
                ("id".into(), Record::new("a", 1).into()),
            ])),
            Response::Fields(BTreeMap::from([
                ("a".into(), 2.into()),
                ("id".into(), Record::new("a", 2).into()),
            ])),
            Response::Fields(BTreeMap::from([
                ("a".into(), 3.into()),
                ("id".into(), Record::new("a", 3).into()),
            ])),
        ];
        assert_eq!(response.len(), 3);
        correct
            .into_iter()
            .for_each(|r| assert!(response.contains(&r)));
    } */

    #[actix_rt::test]
    async fn test_select_record() {
        let select = Select::new(
            ql::fields::Fields::new(vec![Field::WildCard]),
            Value::Record(Box::new(Record::new("a", 1))),
        );

        let graph = Graph::new().start();
        graph.send(Define::table("a")).await.unwrap();

        let msg = graph::Retrieve::new("a");
        let table = graph.send(msg).await.unwrap().unwrap();
        let _ = table.send(Insert::new(1, vec![("a", 1.into())])).await;
        let _ = table.send(Insert::new(2, vec![("a", 2.into())])).await;

        let response = select.execute(graph).await.unwrap();
        let correct = vec![Response::Fields(BTreeMap::from([
            ("a".into(), 1.into()),
            ("id".into(), Record::new("a", 1).into()),
        ]))];
        assert_eq!(response, correct);
    }
}
