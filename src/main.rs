use actix::Actor;
use actor_graph::{Get, Node, Relate, Walk};
use std::{collections::BTreeMap, thread::sleep, time::Duration};
use value::Record;

#[actix::main]
async fn main() {
    let mut tables = BTreeMap::new();
    let a = vec![
        Node::new(String::from("a"), String::from("1"), vec![]).start(),
        Node::new(String::from("a"), String::from("2"), vec![]).start(),
    ];

    tables.insert("a", a);

    let b = vec![
        Node::new(String::from("b"), String::from("1"), vec![]).start(),
        Node::new(String::from("b"), String::from("2"), vec![]).start(),
    ];

    tables.insert("b", b);

    let a = tables.get("a").unwrap().first().unwrap();
    let b = tables.get("b").unwrap().first().unwrap();
    let _ = a
        .send(Relate::Relate {
            edge: "stuff".to_string(),
            fields: vec![],
            address: b.clone(),
        })
        .await;
    sleep(Duration::from_secs(1));
    let x = a
        .send(Walk(Record {
            table: "b".to_string(),
            id: "1".to_string(),
        }))
        .await
        .unwrap()
        .unwrap();

    for a in tables.get("a").unwrap() {
        let x = a.send(Get).await.unwrap();
        println!("{x:#?}");
    }

    for b in tables.get("b").unwrap() {
        let x = b.send(Get).await.unwrap();
        println!("{x:#?}");
    }

    println!("{x}");
}

mod actor_graph {
    use crate::value::{Record, Value};
    use actix::{
        dev::MessageResponse, Actor, Addr, AsyncContext, Context, Handler, Message, ResponseFuture,
    };
    use std::collections::BTreeMap;
    use uuid::Uuid;

    #[derive(Debug, Clone)]
    pub struct Edge {
        pub id: Record,
        pub fields: BTreeMap<String, Value>,
        pub node_1: (Record, Addr<Node>),
        pub node_2: (Record, Addr<Node>),
    }

    impl Edge {
        pub fn new(
            edge: String,
            to: Record,
            from: Record,
            to_address: Addr<Node>,
            from_address: Addr<Node>,
            fields: Vec<(String, Value)>,
        ) -> Self {
            let uuid = Uuid::new_v4().to_string();
            let id = Record::new(edge, uuid);
            let fields = fields.into_iter().collect();
            Edge {
                id,
                fields,
                node_1: (to, to_address),
                node_2: (from, from_address),
            }
        }
    }

    impl Actor for Edge {
        type Context = Context<Self>;

        fn start(self) -> Addr<Self>
        where
            Self: Actor<Context = Context<Self>>,
        {
            let addr = Context::new().run(self);
            addr.do_send(Configure);
            addr
        }
    }

    #[derive(Message)]
    #[rtype(result = "()")]
    pub struct Configure;

    impl Handler<Configure> for Edge {
        type Result = ();

        fn handle(&mut self, _msg: Configure, ctx: &mut Self::Context) -> Self::Result {
            let id_1 = self.node_1.0.clone();
            let id_2 = self.node_2.0.clone();
            self.node_1.1.do_send(Bind(id_2, ctx.address()));
            self.node_2.1.do_send(Bind(id_1, ctx.address()));
        }
    }

    #[derive(Debug, Clone)]
    pub struct Node {
        pub id: Record,
        pub fields: BTreeMap<String, Value>,
        pub edges: BTreeMap<Record, Addr<Edge>>,
    }

    impl Node {
        pub fn id(&self) -> Record {
            self.id.clone()
        }
        pub fn new(table: String, id: String, fields: Vec<(String, Value)>) -> Self {
            let id = Record::new(table, id);
            let fields = fields.into_iter().collect();
            Node {
                id,
                fields,
                edges: BTreeMap::new(),
            }
        }
    }

    impl Actor for Node {
        type Context = Context<Self>;
    }

    #[derive(Message)]
    #[rtype(result = "()")]
    pub struct Update {
        pub fields: Vec<(String, Value)>,
    }

    impl Handler<Update> for Node {
        type Result = ();

        fn handle(&mut self, Update { fields }: Update, _ctx: &mut Self::Context) -> Self::Result {
            for (field, value) in fields {
                if let Some(val) = self.fields.get_mut(&field) {
                    *val = value;
                } else {
                    self.fields.insert(field, value);
                };
            }
        }
    }

    #[derive(Message)]
    #[rtype(result = "()")]
    pub enum Relate {
        Generate {
            edge: String,
            fields: Vec<(String, Value)>,
            from: Record,
            address: Addr<Node>,
        },
        Relate {
            edge: String,
            fields: Vec<(String, Value)>,
            address: Addr<Node>,
        },
    }

    impl Handler<Relate> for Node {
        type Result = ();

        fn handle(&mut self, msg: Relate, ctx: &mut Self::Context) -> Self::Result {
            match msg {
                Relate::Generate {
                    edge,
                    fields,
                    from,
                    address,
                } => {
                    Edge::new(edge, self.id(), from, ctx.address(), address, fields).start();
                }
                Relate::Relate {
                    edge,
                    fields,
                    address: link,
                } => link.do_send(Relate::Generate {
                    edge,
                    fields,
                    from: self.id(),
                    address: ctx.address(),
                }),
            };
        }
    }

    #[derive(Message)]
    #[rtype(result = "()")]
    pub struct Bind(Record, Addr<Edge>);

    impl Handler<Bind> for Node {
        type Result = ();

        fn handle(&mut self, Bind(id, address): Bind, _ctx: &mut Self::Context) -> Self::Result {
            self.edges.insert(id, address);
        }
    }

    #[derive(Message)]
    #[rtype(result = "GetResponses")]
    pub struct Get;

    #[derive(Debug)]
    pub enum GetResponses {
        Node(Node),
        Edge(Edge),
    }

    impl From<Node> for GetResponses {
        fn from(value: Node) -> Self {
            GetResponses::Node(value)
        }
    }

    impl From<Edge> for GetResponses {
        fn from(value: Edge) -> Self {
            GetResponses::Edge(value)
        }
    }
    impl<A, M> MessageResponse<A, M> for GetResponses
    where
        A: Actor,
        M: Message<Result = GetResponses>,
    {
        fn handle(
            self,
            _ctx: &mut A::Context,
            tx: Option<actix::prelude::dev::OneshotSender<M::Result>>,
        ) {
            if let Some(tx) = tx {
                let _ = tx.send(self);
            }
        }
    }

    impl Handler<Get> for Node {
        type Result = GetResponses;

        fn handle(&mut self, _: Get, _: &mut Self::Context) -> Self::Result {
            self.clone().into()
        }
    }

    impl Handler<Get> for Edge {
        type Result = GetResponses;

        fn handle(&mut self, _: Get, _: &mut Self::Context) -> Self::Result {
            self.clone().into()
        }
    }

    #[derive(Message)]
    #[rtype(result = "Result<String, i32>")]
    pub struct Walk(pub Record);

    impl Handler<Walk> for Node {
        type Result = ResponseFuture<Result<String, i32>>;

        fn handle(&mut self, Walk(id): Walk, _ctx: &mut Self::Context) -> Self::Result {
            let x = self.edges.get(&id).unwrap().clone();
            Box::pin(async move {
                let result = x.send(Test).await;
                match result {
                    Ok(Ok(response)) => Ok(format!("Actor 1 got it: {response}")),
                    _ => Err(-1),
                }
            })
        }
    }

    #[derive(Message)]
    #[rtype(result = "Result<String, i32>")]
    pub struct Test;

    impl Handler<Test> for Edge {
        type Result = Result<String, i32>;

        fn handle(&mut self, _msg: Test, _ctx: &mut Self::Context) -> Self::Result {
            Ok("it worked".to_string())
        }
    }
}

mod value {
    use std::collections::BTreeMap;

    #[derive(Debug, Clone, Default)]
    pub enum Value {
        Number(Number),
        String(String),
        Bool(bool),
        Array(Vec<Value>),
        Object(BTreeMap<String, Value>),
        #[default]
        Null,
    }

    impl From<serde_json::Value> for Value {
        fn from(value: serde_json::Value) -> Self {
            match value {
                serde_json::Value::Null => Value::Null,
                serde_json::Value::Bool(bool) => Value::Bool(bool),
                serde_json::Value::Number(num) => Value::from(num),
                serde_json::Value::String(st) => Value::String(st),
                serde_json::Value::Array(vals) => {
                    Value::Array(vals.into_iter().map(Into::into).collect())
                }
                serde_json::Value::Object(obj) => {
                    Value::Object(obj.into_iter().map(|(k, v)| (k, v.into())).collect())
                }
            }
        }
    }

    impl From<serde_json::Number> for Value {
        fn from(value: serde_json::Number) -> Self {
            if let Some(num) = value.as_i64() {
                Value::Number(Number::Int(num))
            } else if let Some(num) = value.as_u64() {
                Value::Number(Number::Int(num as i64))
            } else {
                Value::Number(Number::Float(value.as_f64().unwrap()))
            }
        }
    }

    impl<T> From<Option<T>> for Value
    where
        Value: From<T>,
    {
        fn from(value: Option<T>) -> Self {
            match value {
                Some(value) => Value::from(value),
                None => Value::Null,
            }
        }
    }

    impl From<String> for Value {
        fn from(value: String) -> Self {
            Value::String(value)
        }
    }

    impl From<bool> for Value {
        fn from(value: bool) -> Self {
            Value::Bool(value)
        }
    }

    impl From<i32> for Value {
        fn from(value: i32) -> Self {
            Value::Number(value.into())
        }
    }

    #[derive(Debug, Clone)]
    pub enum Number {
        Int(i64),
        Float(f64),
    }

    impl From<f32> for Number {
        fn from(value: f32) -> Self {
            Number::Float(value as f64)
        }
    }

    impl From<f64> for Number {
        fn from(value: f64) -> Self {
            Number::Float(value)
        }
    }

    impl From<i8> for Number {
        fn from(value: i8) -> Self {
            Number::Int(value as i64)
        }
    }

    impl From<i16> for Number {
        fn from(value: i16) -> Self {
            Number::Int(value as i64)
        }
    }

    impl From<i32> for Number {
        fn from(value: i32) -> Self {
            Number::Int(value as i64)
        }
    }

    impl From<i64> for Number {
        fn from(value: i64) -> Self {
            Number::Int(value)
        }
    }

    impl From<u8> for Number {
        fn from(value: u8) -> Self {
            Number::Int(value as i64)
        }
    }

    impl From<u16> for Number {
        fn from(value: u16) -> Self {
            Number::Int(value as i64)
        }
    }

    impl From<u32> for Number {
        fn from(value: u32) -> Self {
            Number::Int(value as i64)
        }
    }

    impl From<u64> for Number {
        fn from(value: u64) -> Self {
            Number::Int(value as i64)
        }
    }

    #[derive(Debug, Hash, Clone, PartialEq, Eq, Ord, PartialOrd)]
    pub struct Record {
        pub table: String,
        pub id: String,
    }

    impl Record {
        pub fn new(table: String, id: String) -> Self {
            Record { table, id }
        }
    }
}

mod builder {
    use serde_json::Value;

    enum Statement {
        Create(Create),
        Relate(Relate),
    }

    pub struct Relate {
        pub origin: Record,
        pub vertex: Record,
    }

    pub struct Record {
        pub table: String,
        pub id: String,
    }

    impl Relate {
        pub fn relate(origin: Record, vertex: Record) -> Self {
            Relate { origin, vertex }
        }
    }

    pub struct Create {
        pub table: String,
        pub id: String,
        pub fields: Vec<(String, Value)>,
    }

    impl Create {
        pub fn new(table: String, id: String) -> Self {
            Create {
                table,
                id,
                fields: vec![],
            }
        }
        pub fn fields<T>(&mut self, fields: Vec<(String, T)>)
        where
            Value: From<T>,
        {
            self.fields.append(
                &mut fields
                    .into_iter()
                    .map(|(id, e)| (id, Value::from(e)))
                    .collect(),
            )
        }

        pub fn field<T>(&mut self, id: String, value: T)
        where
            Value: From<T>,
        {
            self.fields.push((id, Value::from(value)))
        }
    }

    enum Syntax {}
}
