use crate::{
    dbs::{edge::Edge, node::Node, ops::get},
    ql::record::Record,
};
use actix::{Handler, MailboxError, Message, ResponseFuture};
use std::sync::Arc;

#[derive(Message)]
#[rtype(result = "Result<Vec<get::Response>, MailboxError>")]
pub struct Walk(pub Vec<Arc<str>>, pub Arc<str>);

impl Walk {
    pub fn new<W: Into<Arc<str>>, T: Into<Arc<str>>>(path: Vec<W>, field: T) -> Self {
        let path = path.into_iter().map(Into::into).collect();
        Walk(path, field.into())
    }
}

impl Handler<Walk> for Node {
    type Result = ResponseFuture<Result<Vec<get::Response>, MailboxError>>;

    fn handle(&mut self, walk: Walk, _ctx: &mut Self::Context) -> Self::Result {
        let Walk(mut path, field) = walk;
        let Some(edge) = path.pop() else {
            let fields = if &*field == "*" {
                self.fields().into()
            } else if &*field == "id" {
                self.id().into()
            } else if let Some(value) = self.fields.get(&field) {
                value.clone().into()
            } else {
                get::Response::None
            };

            return Box::pin(async move { Ok(vec![fields]) });
        };
        let edges = self
            .edges
            .iter()
            .filter_map(|(Record { table, id: _ }, addr)| {
                if *table == edge {
                    Some(addr.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        Box::pin(async move {
            let mut responses = Vec::new();
            for edge in edges {
                let resp = edge.send(Walk::new(path.clone(), field.clone())).await??;
                responses.push(resp);
            }
            Ok(responses.into_iter().flatten().collect())
        })
    }
}

impl Handler<Walk> for Edge {
    type Result = ResponseFuture<Result<Vec<get::Response>, MailboxError>>;

    fn handle(&mut self, Walk(mut path, field): Walk, _ctx: &mut Self::Context) -> Self::Result {
        let Some(table) = path.pop() else {
            let fields = if &*field == "*" {
                self.fields().into()
            } else if &*field == "id" {
                self.id().into()
            } else if let Some(value) = self.fields.get(&field) {
                value.clone().into()
            } else {
                get::Response::None
            };

            return Box::pin(async move { Ok(vec![fields]) });
        };
        if self.dest.id.table != table && &*table != "?" {
            return Box::pin(async move { Ok(Vec::new()) });
        }
        let edge = self.dest.address();
        Box::pin(async move { edge.send(Walk::new(path.clone(), field)).await? })
    }
}

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;

    use super::*;
    use crate::{dbs::ops::relate::Relate, ql::value::Value};
    use actix::Actor;

    #[actix::test]
    async fn test_walk() {
        let fields: Vec<(Arc<str>, _)> = Vec::new();
        let a = Node::new(Record::new("a", "1"), fields.clone()).start();
        let b = Node::new(Record::new("b", "2"), fields.clone()).start();

        let _ = a
            .send(Relate::Relate {
                edge: "e_1".to_string(),
                fields: vec![],
                address: b.clone(),
            })
            .await;
        let mut path = vec!["e_1", "b"];
        path.reverse();
        let mut res = a.send(Walk::new(path, "*")).await.unwrap().unwrap();

        let res = res.pop().unwrap();
        let res: BTreeMap<Arc<str>, Value> = res.try_into().unwrap();
        let value = res.get("id".into()).unwrap();
        assert_eq!(*value, Value::Record(Box::new(Record::new("b", "2"))))
    }

    #[actix::test]
    async fn test_walk_two() {
        let fields: Vec<(Arc<str>, _)> = Vec::new();
        let a = Node::new(Record::new("a", "1"), fields.clone()).start();
        let b = Node::new(Record::new("b", "2"), fields.clone()).start();
        let c = Node::new(Record::new("c", "3"), fields.clone()).start();

        let _ = a
            .send(Relate::Relate {
                edge: "e_1".to_string(),
                fields: vec![],
                address: b.clone(),
            })
            .await;

        let _ = b
            .send(Relate::Relate {
                edge: "e_2".to_string(),
                fields: vec![],
                address: c.clone(),
            })
            .await;
        let mut path = vec!["e_1", "b", "e_2", "c"];
        path.reverse();

        let mut res = a.send(Walk::new(path, "id")).await.unwrap().unwrap();

        let get = res.pop().unwrap();
        let value: Value = get.try_into().unwrap();
        assert_eq!(value, Value::Record(Box::new(Record::new("c", "3"))))
    }

    #[actix::test]
    async fn test_walk_empty() {
        let fields: Vec<(Arc<str>, _)> = Vec::new();
        let a = Node::new(Record::new("a", "1"), fields.clone()).start();
        let b = Node::new(Record::new("b", "2"), fields.clone()).start();
        let c = Node::new(Record::new("c", "3"), fields.clone()).start();

        let _ = a
            .send(Relate::Relate {
                edge: "e_1".to_string(),
                fields: vec![],
                address: b.clone(),
            })
            .await;

        let _ = b
            .send(Relate::Relate {
                edge: "e_2".to_string(),
                fields: vec![],
                address: c.clone(),
            })
            .await;
        let mut path = vec!["e_1", "z", "e_2", "c"];
        path.reverse();

        let res = a.send(Walk::new(path, "*")).await.unwrap().unwrap();
        assert!(res.is_empty())
    }

    #[actix::test]
    async fn walk_field_test() {
        let fields: Vec<(Arc<str>, _)> = Vec::new();
        let a = Node::new(Record::new("a", "1"), fields.clone()).start();
        let b = Node::new(Record::new("b", "2"), fields.clone()).start();
        let c = Node::new(Record::new("c", "3"), vec![("car".into(), "1".into())]).start();

        let _ = a
            .send(Relate::Relate {
                edge: "e_1".to_string(),
                fields: vec![],
                address: b.clone(),
            })
            .await;

        let _ = b
            .send(Relate::Relate {
                edge: "e_2".to_string(),
                fields: vec![],
                address: c.clone(),
            })
            .await;
        let mut path = vec!["e_1", "b", "e_2", "c"];
        path.reverse();

        let mut res = a.send(Walk::new(path, "car")).await.unwrap().unwrap();

        let x = res.pop().unwrap();
        let x: Value = x.try_into().unwrap();
        assert_eq!(x, Value::String("1".into()))
    }
}
