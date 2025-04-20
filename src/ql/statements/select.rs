use crate::{
    dbs::{
        graph::Graph,
        iterator::{Iterable, Iterator},
    },
    doc::document::Cursor,
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
use reblessive::tree::Stk;

#[non_exhaustive]
pub struct Select {
    pub fields: Fields,
    pub what: Values,
    pub conditions: Option<Condition>,
    pub limit: Option<usize>,
    pub start: Option<usize>,
}

impl Select {
    // TODO need to implement multi step edges
    async fn compute(
        &self,
        stk: &mut Stk,
        graph: Addr<Graph>,
        cur: Option<&Cursor>,
    ) -> Result<Value, Error> {
        let mut iter = Iterator::new();
        let stm = Statement::from(self);
        for val in self.what.0.iter() {
            let val = stk.run(|stk| val.evaluate(stk, &graph, cur)).await?;
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

        let result = stk.run(|stk| iter.process(stk, &graph, &stm)).await;

        result
    }
}

/* #[cfg(test)]
mod test {
    use super::*;
    use crate::{
        dbs::{
            self, graph,
            ops::{define::Define, insert::Insert},
        },
        ql::{fields::Field, ident::Ident, object::Object, record::Record, table::Table},
    };
    use actix::Actor;

    async fn generate_graph(
        name: String,
        count: usize,
    ) -> (Addr<Graph>, Vec<Addr<dbs::table::Table>>) {
        let graph = Graph::new().start();
        let _ = graph.send(Define::Table(name.clone())).await.unwrap();

        let retrieve = graph::Retrieve(Table(name).into());
        let table = graph.send(retrieve).await.unwrap().unwrap();

        for i in 0..count {
            let i = i as i32;
            let insert = Insert(
                Record::generate("a").into(),
                vec![("count".into(), i.into()), ("b".into(), Value::Null)],
            );
            table.send(insert).await.unwrap().unwrap();
        }

        (graph, vec![table])
    }

    #[actix_rt::test]
    async fn select_wildcard_test() {
        let length = 3;
        let (graph, _tables) = generate_graph("a".to_string(), length).await;

        let select = Select {
            fields: Fields(vec![Field::WildCard]),
            what: Values(vec![Table(String::from("a")).into()]),
            conditions: None,
            limit: None,
            start: None,
        };
        let res: Vec<Value> = select.compute(graph).await.unwrap().try_into().unwrap();

        assert_eq!(res.len(), length)
    }

    #[actix_rt::test]
    async fn select_limit_test() {
        let limit = 1;
        let (graph, _tables) = generate_graph("a".to_string(), 3).await;

        let select = Select {
            fields: Fields(vec![Field::WildCard]),
            what: Values(vec![Table(String::from("a")).into()]),
            conditions: None,
            limit: Some(limit),
            start: None,
        };
        let res: Vec<Value> = select.compute(graph).await.unwrap().try_into().unwrap();

        assert_eq!(res.len(), limit);
    }

    #[actix_rt::test]
    async fn select_start_test() {
        let length = 3;
        let (graph, _tables) = generate_graph("a".to_string(), length).await;

        let select = Select {
            fields: Fields(vec![Field::Single {
                expr: Ident("count".into()).into(),
                alias: None,
            }]),
            what: Values(vec![Table(String::from("a")).into()]),
            conditions: None,
            limit: None,
            start: Some(1),
        };
        let res: Vec<Value> = select.compute(graph).await.unwrap().try_into().unwrap();

        for val in res.iter() {
            let Value::Object(Object(obj)) = val else {
                panic!("object type wasnt returned: {val}");
            };
            assert_eq!(obj.len(), 1);
            obj.get("count").unwrap();
        }

        assert_eq!(res.len(), 2)
    }

    #[actix_rt::test]
    async fn select_count_test() {
        let length = 3;
        let (graph, _tables) = generate_graph("a".to_string(), length).await;

        let select = Select {
            fields: Fields(vec![Field::Single {
                expr: Ident("count".into()).into(),
                alias: None,
            }]),
            what: Values(vec![Table(String::from("a")).into()]),
            conditions: None,
            limit: None,
            start: None,
        };
        let res: Vec<Value> = select.compute(graph).await.unwrap().try_into().unwrap();

        for val in res.iter() {
            let Value::Object(Object(obj)) = val else {
                panic!("object type wasnt returned: {val}");
            };
            assert_eq!(obj.len(), 1);
            obj.get("count").unwrap();
        }

        assert_eq!(res.len(), length)
    }

    #[actix_rt::test]
    async fn select_two_fields_test() {
        let length = 3;
        let (graph, _tables) = generate_graph("a".to_string(), length).await;

        let select = Select {
            fields: Fields(vec![
                Field::Single {
                    expr: Ident("count".into()).into(),
                    alias: None,
                },
                Field::Single {
                    expr: Ident("a".into()).into(),
                    alias: Some("test".to_string()),
                },
            ]),
            what: Values(vec![Table(String::from("a")).into()]),
            conditions: None,
            limit: None,
            start: None,
        };
        let res: Vec<Value> = select.compute(graph).await.unwrap().try_into().unwrap();

        for val in res.iter() {
            let Value::Object(Object(obj)) = val else {
                panic!("object type wasnt returned: {val}");
            };
            assert_eq!(obj.len(), 2);
            obj.get("count").unwrap();
            obj.get("test").unwrap();
        }

        assert_eq!(res.len(), length)
    }
} */
