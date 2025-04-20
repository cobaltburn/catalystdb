use crate::{
    dbs::{
        self,
        entity::Entity,
        graph::{self, Graph},
        ops::{get::Get, walk::Walk},
        table,
    },
    err::Error,
    ql::{
        edge::Edge, fields::Field, ident::Ident, idiom::Idioms, object::Object, path::Path,
        record::Record, statements::statement::Statement, table::Table, value::Value,
    },
    resp::Response,
};
use actix::Addr;
use dbs::ops::retrieve::Retrieve;
use reblessive::tree::Stk;
use std::{collections::BTreeMap, mem, sync::Arc};

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Iterable {
    Value(Value),
    Edge(Edge),
    Record(Addr<Entity>),
    Table(Vec<Addr<Entity>>),
}

impl Iterable {
    pub async fn process(
        self,
        stk: &mut Stk,
        graph: &Addr<Graph>,
        stm: &Statement<'_>,
    ) -> Result<Value, Error> {
        match self {
            Iterable::Value(value) => Self::process_value(value, stk, graph, stm).await,
            Iterable::Edge(edge) => Self::process_edge(edge, graph).await,
            Iterable::Record(record) => Self::process_record(record, stm).await,
            Iterable::Table(table) => Self::process_table(table, stm).await,
        }
    }

    async fn process_edge(
        Edge { dir, from, to }: Edge,
        graph: &Addr<Graph>,
    ) -> Result<Value, Error> {
        let response = graph.send(Retrieve::Record(from.clone())).await.unwrap();

        let node = match response {
            Response::Record(addr) => addr,
            Response::None => return Ok(Value::None),
            _ => unreachable!(),
        };

        let mut responses: Vec<Value> = Vec::new();

        let walk = Walk::new(
            vec![Path::new(dir.clone(), to, None)],
            from.clone(),
            Field::Single {
                expr: Ident("id".into()).into(),
                alias: None,
            },
            graph.clone(),
        );
        let response = node.send(walk).await.unwrap()?;
        responses.push(response.try_into()?);

        Ok(responses.into())
    }

    async fn process_value(
        value: Value,
        stk: &mut Stk,
        graph: &Addr<Graph>,
        stm: &Statement<'_>,
    ) -> Result<Value, Error> {
        let fields = stm.fields().ok_or(Error::InvalidStatement())?;
        let mut object: BTreeMap<Arc<str>, Value> = BTreeMap::new();
        for field in &fields.0 {
            match field {
                Field::WildCard => {
                    object.insert(value.to_string().into(), value.clone());
                }
                Field::Single { expr, alias } => {
                    // TODO need to fix whats going on here
                    // let key = alias.clone().map_or(expr.to_string().into(), Into::into);
                    // let value = stk
                    //     .run(|stk| value.evaluate(stk, &expr, graph))
                    //     .unwrap_or(Value::None);
                    // object.insert(key, value);
                }
            }
        }
        Ok(Value::Object(Object(object)))
    }

    async fn process_record(record: Addr<Entity>, stm: &Statement<'_>) -> Result<Value, Error> {
        let response = match stm {
            Statement::Select(stm) => record
                .send(Get::new(stm.fields.clone(), stm.conditions.clone()))
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
                    let filter = stm.conditions.clone();
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
    pub limit: Option<usize>,
    pub start: Option<usize>,
    pub entries: Vec<Iterable>,
}

impl Iterator {
    pub fn new() -> Iterator {
        Iterator::default()
    }

    pub fn ingest(&mut self, value: Iterable) {
        self.entries.push(value);
    }

    pub async fn ingest_record(&mut self, id: Record, graph: &Addr<Graph>) -> Result<(), Error> {
        let response = graph.send(Retrieve::Record(id.clone())).await.unwrap();

        let node = match response {
            Response::Record(addr) => addr,
            Response::None => return Ok(()),
            _ => unreachable!(),
        };

        self.ingest(Iterable::Record(node));

        Ok(())
    }

    pub async fn ingest_table(
        &mut self,
        Table(table): Table,
        graph: &Addr<Graph>,
    ) -> Result<(), Error> {
        let retrieve = Retrieve::Table(table);
        let response = graph.send(retrieve).await.unwrap();
        let table = match response {
            Response::Table(table) => table,
            Response::None => return Ok(()),
            _ => unreachable!(),
        };

        self.ingest(Iterable::Table(table));

        Ok(())
    }

    pub fn ingest_edge(&mut self, edge: Edge) -> Result<(), Error> {
        self.ingest(Iterable::Edge(edge));
        Ok(())
    }

    pub fn set_limit(&mut self, stm: &Statement<'_>) -> Result<(), Error> {
        self.limit = stm.limit().map(Clone::clone);

        Ok(())
    }

    pub fn set_start(&mut self, stm: &Statement<'_>) -> Result<(), Error> {
        self.start = stm.start().map(Clone::clone);

        Ok(())
    }

    pub async fn process(
        &mut self,
        stk: &mut Stk,
        graph: &Addr<Graph>,
        stm: &Statement<'_>,
    ) -> Result<Value, Error> {
        self.set_start(stm)?;
        self.set_limit(stm)?;

        let mut values = vec![];
        for val in mem::take(&mut self.entries) {
            let val = val.process(stk, graph, stm).await?;
            values.push(val);
        }

        let values: Value = values.into();
        let mut values: Vec<Value> = values.flatten().try_into()?;

        if let Some(i) = self.start {
            values = values.into_iter().skip(i).collect();
        }

        if let Some(i) = self.limit {
            values = values.into_iter().take(i).collect();
        }

        Ok(values.into())
    }
}

#[cfg(test)]
mod test {}
