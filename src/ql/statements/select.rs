/* use crate::{
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
} */

#[cfg(test)]
mod test {}
