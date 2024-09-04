use crate::ql::{record::Record, value::Value};
use actix::{Actor, Addr, Context, Handler, Message, ResponseFuture, SyncContext};
use get::{FieldIds, Get, GetResponses};
use node::Node;
use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

pub mod delete;
pub mod edge;
pub mod get;
pub mod node;
pub mod relate;
pub mod remove;
pub mod update;
pub mod walk;

#[derive(Debug, Default)]
pub struct Graph {
    table: BTreeMap<String, Addr<Table>>,
}

impl Actor for Graph {
    type Context = SyncContext<Self>;
}

impl Graph {
    pub fn new() -> Self {
        Graph::default()
    }
}

#[derive(Message)]
#[rtype(result = "Option<Addr<Table>>")]
pub struct GetTable(String);

impl GetTable {
    pub fn new(table: String) -> Self {
        GetTable(table)
    }
}

impl Handler<GetTable> for Graph {
    type Result = Option<Addr<Table>>;

    fn handle(&mut self, GetTable(table): GetTable, _ctx: &mut Self::Context) -> Self::Result {
        self.table.get(&table).map(|e| e.clone())
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct NewTable(pub String);

impl NewTable {
    pub fn new(table: String) -> Self {
        NewTable(table)
    }
}

impl Handler<NewTable> for Graph {
    type Result = ();

    fn handle(&mut self, NewTable(table): NewTable, _ctx: &mut Self::Context) -> Self::Result {
        if !self.table.contains_key(&table) {
            self.table.insert(table, Table::spawn());
        }
    }
}

type Row = (Record, Addr<Node>);

#[derive(Debug, Default)]
pub struct Table {
    pub nodes: Vec<Row>,
    pub keys: HashMap<Record, usize>,
}

impl Actor for Table {
    type Context = Context<Self>;
}

impl Table {
    fn new() -> Self {
        Table::default()
    }

    fn spawn() -> Addr<Self> {
        Table::default().start()
    }

    fn insert(&mut self, record: Record, addr: Addr<Node>) {
        self.nodes.push((record.clone(), addr));
        self.keys.insert(record, self.nodes.len() - 1);
    }

    fn contains(&self, record: &Record) -> bool {
        self.keys.contains_key(record)
    }

    fn get(&self, id: Record) -> Option<Addr<Node>> {
        let index = self.keys.get(&id)?;
        let (_, node) = self.nodes.get(*index)?;
        Some(node.clone())
    }
}

#[derive(Message)]
#[rtype(result = "Result<(), String>")]
pub struct Insert(Record, Vec<(Arc<str>, Value)>);

impl Insert {
    pub fn new<R: Into<Record>, W: Into<Arc<str>>>(record: R, fields: Vec<(W, Value)>) -> Self {
        let record = record.into();
        let fields = fields.into_iter().map(|(id, v)| (id.into(), v)).collect();
        Insert(record, fields)
    }
}

impl Handler<Insert> for Table {
    type Result = Result<(), String>;

    fn handle(&mut self, Insert(id, fields): Insert, _ctx: &mut Self::Context) -> Self::Result {
        if !self.contains(&id) {
            let addr = Node::spawn(id.clone(), fields);
            self.insert(id, addr);
            return Ok(());
        }
        Err(String::from("Record already exists"))
    }
}

#[derive(Message)]
#[rtype(result = "Option<GetResponses>")]
pub struct Select {
    id: Record,
    fields: Vec<Arc<str>>,
}

impl Select {
    pub fn new<R: Into<Record>, S: Into<Arc<str>>>(id: R, fields: Vec<S>) -> Self {
        Select {
            id: id.into(),
            fields: fields.into_iter().map(Into::into).collect(),
        }
    }
}

impl Handler<Select> for Table {
    type Result = ResponseFuture<Option<GetResponses>>;

    fn handle(&mut self, Select { id, fields }: Select, _ctx: &mut Self::Context) -> Self::Result {
        let addr = self.get(id);
        Box::pin(async move { addr?.send(Get(FieldIds::Fields(fields))).await.ok() })
    }
}

#[cfg(test)]
mod test {
    use std::{thread, time::Duration};

    use actix::SyncArbiter;

    use super::*;

    #[actix_rt::test]
    async fn test_graph_table() {
        // TODO need to fix this test
        let addr = SyncArbiter::start(1, || Graph::new());

        addr.send(NewTable(String::from("a"))).await.unwrap();

        addr.send(GetTable(String::from("a")))
            .await
            .unwrap()
            .unwrap();
        let a = addr.send(GetTable(String::from("b"))).await.unwrap();
        assert!(a.is_none());
    }
}
