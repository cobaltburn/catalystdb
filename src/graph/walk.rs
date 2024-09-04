use crate::{
    graph::{edge::Edge, get::GetResponses, node::Node},
    ql::record::Record,
};
use actix::{Handler, MailboxError, Message, ResponseFuture};
use std::sync::Arc;

#[derive(Message)]
#[rtype(result = "Result<Vec<GetResponses>, MailboxError>")]
pub struct Walk(pub Vec<Arc<str>>, pub Arc<str>);

impl Handler<Walk> for Node {
    type Result = ResponseFuture<Result<Vec<GetResponses>, MailboxError>>;

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
                GetResponses::None
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
                let resp = edge.send(Walk(path.clone(), field.clone())).await??;
                responses.push(resp);
            }
            Ok(responses.into_iter().flatten().collect())
        })
    }
}

impl Handler<Walk> for Edge {
    type Result = ResponseFuture<Result<Vec<GetResponses>, MailboxError>>;

    fn handle(&mut self, Walk(mut path, field): Walk, _ctx: &mut Self::Context) -> Self::Result {
        let Some(table) = path.pop() else {
            let fields = if &*field == "*" {
                self.fields().into()
            } else if &*field == "id" {
                self.id().into()
            } else if let Some(value) = self.fields.get(&field) {
                value.clone().into()
            } else {
                GetResponses::None
            };

            return Box::pin(async move { Ok(vec![fields]) });
        };
        if self.node_2.id.table != table && &*table != "?" {
            return Box::pin(async move { Ok(Vec::new()) });
        }
        let edge = self.node_2.address();
        Box::pin(async move { edge.send(Walk(path.clone(), field)).await? })
    }
}

#[cfg(test)]
mod test {
    use actix::Actor;

    use crate::{graph::relate::Relate, ql::value::Value};

    use super::*;

    #[actix::test]
    async fn walk_test() {
        let a = Node::new(("a", "1").into(), vec![]).start();
        let b = Node::new(("b", "2").into(), vec![]).start();

        let _ = a
            .send(Relate::Relate {
                edge: "e_1".to_string(),
                fields: vec![],
                address: b.clone(),
            })
            .await;
        let mut path = vec!["e_1", "b"]
            .into_iter()
            .map(Into::into)
            .collect::<Vec<_>>();
        path.reverse();
        let mut res = a.send(Walk(path, "*".into())).await.unwrap().unwrap();

        let x = res.pop().unwrap();
        let mut x: Vec<(Arc<str>, Value)> = x.try_into().unwrap();
        let (_, x) = x.pop().unwrap();
        assert_eq!(x, Value::Record(Record::new("b", "2")))
    }

    #[actix::test]
    async fn walk_two_test() {
        let a = Node::new(("a", "1").into(), vec![]).start();
        let b = Node::new(("b", "2").into(), vec![]).start();
        let c = Node::new(("c", "3").into(), vec![]).start();

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
        let mut path = vec!["e_1", "b", "e_2", "c"]
            .into_iter()
            .map(Into::into)
            .collect::<Vec<_>>();
        path.reverse();

        let mut res = a.send(Walk(path, "id".into())).await.unwrap().unwrap();

        let get = res.pop().unwrap();
        let value: Value = get.try_into().unwrap();
        assert_eq!(value, Value::Record(Record::new("c", "3")))
    }

    #[actix::test]
    async fn walk_empty_test() {
        let a = Node::new(("a", "1").into(), vec![]).start();
        let b = Node::new(("b", "2").into(), vec![]).start();
        let c = Node::new(("c", "3").into(), vec![]).start();

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
        let mut path = vec!["e_1", "z", "e_2", "c"]
            .into_iter()
            .map(Into::into)
            .collect::<Vec<_>>();
        path.reverse();

        let res = a.send(Walk(path, "*".into())).await.unwrap().unwrap();
        assert!(res.is_empty())
    }

    #[actix::test]
    async fn walk_field_test() {
        let a = Node::spawn(("a", "1").into(), vec![]);
        let b = Node::spawn(("b", "2").into(), vec![]);
        let c = Node::spawn(("c", "3").into(), vec![("car".into(), "1".into())]);

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
        let mut path = vec!["e_1", "b", "e_2", "c"]
            .into_iter()
            .map(Into::into)
            .collect::<Vec<_>>();
        path.reverse();

        let mut res = a.send(Walk(path, "car".into())).await.unwrap().unwrap();

        let x = res.pop().unwrap();
        let x: Value = x.try_into().unwrap();
        assert_eq!(x, Value::String("1".into()))
    }
}
