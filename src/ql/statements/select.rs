use crate::{
    dbs::{
        graph::Graph,
        iterator::{Iterable, Iterator},
    },
    err::Error,
    ql::{
        array::Array,
        condition::Condition,
        fields::Fields,
        statements::statement::Statement,
        value::{Value, Values},
    },
};
use actix::Addr;

#[non_exhaustive]
pub struct Select {
    pub fields: Fields,
    pub what: Values,
    pub filter: Option<Condition>,
    pub limit: Option<u32>,
    pub start: Option<u32>,
}

impl Select {
    async fn evaluate(&self, graph: Addr<Graph>) -> Result<Value, Error> {
        let mut iter = Iterator::new();
        let stm = Statement::from(self);
        for val in self.what.0.iter() {
            let val = val.evaluate(&Value::None)?;
            match val {
                Value::Record(id) => iter.ingest_record(*id, &graph).await?,
                Value::Table(table) => iter.ingest_table(table, &graph).await?,
                Value::Edge(edge) => iter.ingest_edge(*edge)?,
                Value::Array(Array(array)) => {
                    for val in array {
                        match val {
                            Value::Record(id) => iter.ingest_record(*id, &graph).await?,
                            Value::Edge(edge) => iter.ingest_edge(*edge)?,
                            Value::Table(table) => iter.ingest_table(table, &graph).await?,
                            _ => iter.ingest(Iterable::Value(val)),
                        }
                    }
                }
                _ => iter.ingest(Iterable::Value(val)),
            }
        }
        let result = iter.process(&graph, &stm).await?;

        todo!()
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use actix::Actor;

    #[actix_rt::test]
    async fn select_test() {
        let graph = Graph::new().start();
        let x = Select {
            fields: Fields(vec![]),
            what: Values(vec![]),
            filter: None,
            limit: None,
            start: None,
        };
        let x = x.evaluate(graph).await.unwrap();
    }
}
