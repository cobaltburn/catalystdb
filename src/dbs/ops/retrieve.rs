use crate::{
    dbs::{graph::Graph, table::Table},
    ql::record::Record,
    resp::Response,
};
use actix::{Addr, Handler, Message, ResponseFuture};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

#[derive(Message)]
#[rtype(result = "Response")]
pub enum Retrieve {
    Table(String),
    Record(Record),
}

impl Retrieve {
    pub fn table<'a>(&self, graph: &'a Graph) -> Option<&'a Addr<Table>> {
        Some(match self {
            Retrieve::Record(record) => graph.tables.get(&record.table.to_string())?,
            Retrieve::Table(table) => graph.tables.get(table)?,
        })
    }
}

impl Handler<Retrieve> for Graph {
    type Result = ResponseFuture<Response>;

    fn handle(&mut self, retrieve: Retrieve, _ctx: &mut Self::Context) -> Self::Result {
        let table = retrieve.table(self);
        let Some(table) = table else {
            return Box::pin(async { Response::None });
        };
        let table = table.clone();

        Box::pin(async move { table.send(retrieve).await.unwrap() })
    }
}

impl Handler<Retrieve> for Table {
    type Result = ResponseFuture<Response>;

    fn handle(&mut self, msg: Retrieve, _ctx: &mut Self::Context) -> Self::Result {
        let nodes = self.nodes.clone();
        Box::pin(async move {
            match msg {
                Retrieve::Table(_) => nodes
                    .read()
                    .unwrap()
                    .par_iter()
                    .map(|(_, addr)| addr.clone())
                    .collect::<Vec<_>>()
                    .into(),
                Retrieve::Record(Record { table: _, id }) => nodes
                    .read()
                    .unwrap()
                    .get(&id)
                    .map_or(Response::None, |addr| Response::Record(addr.clone())),
            }
        })
    }
}
