use crate::{
    dbs::{
        self,
        entity::Entity,
        graph::{self, Graph},
        ops::{
            get::Get,
            walk::{Step, Walk},
        },
        table,
    },
    err::Error,
    ql::{
        edge::Edge, fields::Field, ident::Ident, object::Object, record::Record,
        statements::statement::Statement, table::Table, value::Value,
    },
    resp::Response,
};
use actix::Addr;
use std::{collections::BTreeMap, mem, sync::Arc};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Iterable {
    Value(Value),
    Edge(Edge),
    Record(Addr<Entity>),
    Table(Vec<Addr<Entity>>),
}

impl Iterable {
    pub async fn process(self, graph: &Addr<Graph>, stm: &Statement<'_>) -> Result<Value, Error> {
        match self {
            Iterable::Value(value) => Self::process_value(value, stm),
            Iterable::Edge(edge) => Self::process_edge(edge, graph).await,
            Iterable::Record(record) => Self::process_record(record, stm).await,
            Iterable::Table(table) => Self::process_table(table, stm).await,
        }
    }

    async fn process_edge(
        Edge {
            dir,
            from,
            to: tables,
        }: Edge,
        graph: &Addr<Graph>,
    ) -> Result<Value, Error> {
        let table = graph
            .send(graph::Retrieve(Value::Record(Box::new(from.clone()))))
            .await
            .unwrap();

        let Some(table) = table else {
            return Ok(Value::None);
        };

        let retrieve = table::Retrieve::Record(from.clone());
        let node: Addr<Entity> = table.send(retrieve).await.unwrap().try_into()?;
        let mut responses: Vec<Value> = Vec::new();

        for table in tables.0 {
            let walk = Walk::new(
                vec![Step::new(dir.clone(), table, None)],
                from.clone(),
                Arc::new(Field::Single {
                    expr: Ident("id".into()).into(),
                    alias: None,
                }),
            );
            let response = node.send(walk).await.unwrap()?;
            responses.push(response.try_into()?);
        }
        Ok(responses.into())
    }

    fn process_value(value: Value, stm: &Statement<'_>) -> Result<Value, Error> {
        let fields = stm.fields().ok_or(Error::InvalidStatement())?;
        let mut object: BTreeMap<Arc<str>, Value> = BTreeMap::new();
        for field in &fields.0 {
            match field {
                Field::WildCard => {
                    object.insert(value.to_string().into(), value.clone());
                }
                Field::Single { expr, alias } => {
                    let key = alias.clone().map_or(expr.to_string().into(), Into::into);
                    let value = value.evaluate(&expr).unwrap_or(Value::None);
                    object.insert(key, value);
                }
            }
        }
        Ok(Value::Object(Object(object)))
    }

    async fn process_record(record: Addr<Entity>, stm: &Statement<'_>) -> Result<Value, Error> {
        let response = match stm {
            Statement::Select(stm) => record
                .send(Get::new(stm.fields.clone(), stm.filter.clone()))
                .await
                .unwrap()?,
        };
        Ok(match response {
            Response::Value(value) => value,
            Response::None => Value::None,
            _ => unreachable!(),
        })
    }

    async fn process_table(table: Vec<Addr<Entity>>, stm: &Statement<'_>) -> Result<Value, Error> {
        let mut values = vec![];
        match stm {
            Statement::Select(stm) => {
                for addr in table {
                    let fields = stm.fields.clone();
                    let filter = stm.filter.clone();
                    let response = addr.send(Get::new(fields, filter)).await.unwrap()?;
                    let val = match response {
                        Response::Value(value) => value,
                        Response::None => Value::None,
                        _ => unreachable!(),
                    };
                    values.push(val);
                }
            }
        };

        Ok(values.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Iterator {
    pub count: u64,
    pub limit: Option<u32>,
    pub start: Option<u32>,
    pub entries: Vec<Iterable>,
}

impl Iterator {
    pub fn new() -> Iterator {
        Iterator::default()
    }

    pub fn ingest(&mut self, value: Iterable) {
        &self.entries.push(value);
    }

    pub async fn ingest_record(&mut self, id: Record, graph: &Addr<Graph>) -> Result<(), Error> {
        let retrieve = graph::Retrieve(Value::Record(Box::new(id.clone())));
        let table = graph.send(retrieve).await.unwrap().ok_or(Error::None)?;

        let retrieve = dbs::table::Retrieve::Record(id);
        let node: Addr<Entity> = table.send(retrieve).await.unwrap().try_into()?;
        &self.ingest(Iterable::Record(node));

        Ok(())
    }

    pub async fn ingest_table(&mut self, table: Table, graph: &Addr<Graph>) -> Result<(), Error> {
        let retrieve = graph::Retrieve(Value::Table(table));
        let table = graph.send(retrieve).await.unwrap().ok_or(Error::None)?;

        let retrieve = dbs::table::Retrieve::Iterator;
        let table: Vec<Addr<Entity>> = table.send(retrieve).await.unwrap().try_into()?;
        &self.ingest(Iterable::Table(table));

        Ok(())
    }

    pub fn ingest_edge(&mut self, edge: Edge) -> Result<(), Error> {
        self.ingest(Iterable::Edge(edge));
        Ok(())
    }

    /* pub async fn iterate(
        &mut self,
        graph: &Addr<Graph>,
        stm: &Statement<'_>,
    ) -> Result<Value, Error> {
        todo!()
    } */

    pub async fn process(
        &mut self,
        graph: &Addr<Graph>,
        stm: &Statement<'_>,
    ) -> Result<Value, Error> {
        let mut values = vec![];
        for iter in mem::take(&mut self.entries) {
            let val = iter.process(graph, stm).await?;
            values.push(val);
        }
        let values = values;
        Ok(values.into())
    }
}
