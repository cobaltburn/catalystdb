use crate::dbs::{edge::Edge, node::Node};
use actix::{ActorContext, Handler, Message};

#[derive(Message)]
#[rtype(result = "()")]
pub struct Delete;

impl Handler<Delete> for Node {
    type Result = ();

    fn handle(&mut self, _msg: Delete, ctx: &mut Self::Context) -> Self::Result {
        ctx.stop()
    }
}

impl Handler<Delete> for Edge {
    type Result = ();

    fn handle(&mut self, _msg: Delete, ctx: &mut Self::Context) -> Self::Result {
        ctx.stop();
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use super::*;
    use crate::{dbs::ops::relate::Relate, ql::record::Record};
    use actix::{Actor, Addr};

    #[derive(Message)]
    #[rtype(result = "Option<Vec<(Record, Addr<Edge>)>>")]
    pub struct GetEdges;

    impl Handler<GetEdges> for Node {
        type Result = Option<Vec<(Record, Addr<Edge>)>>;

        fn handle(&mut self, _msg: GetEdges, _ctx: &mut Self::Context) -> Self::Result {
            let edges = self.edges.clone().into_iter().collect();
            Some(edges)
        }
    }

    #[actix::test]
    async fn delete_test() {
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
        b.send(Delete).await.unwrap();
        let res = a.send(GetEdges).await.unwrap().unwrap();
        assert!(res.is_empty())
    }

    #[actix::test]
    async fn delete_two_test() {
        let fields: Vec<(Arc<str>, _)> = Vec::new();
        let a = Node::new(Record::new("a", "1"), fields.clone()).start();
        let b = Node::new(Record::new("b", "2"), fields.clone()).start();
        let c = Node::new(Record::new("c", "2"), fields.clone()).start();
        let _ = a
            .send(Relate::Relate {
                edge: "e_1".to_string(),
                fields: vec![],
                address: b.clone(),
            })
            .await;
        let _ = a
            .send(Relate::Relate {
                edge: "e_2".to_string(),
                fields: vec![],
                address: c.clone(),
            })
            .await;
        b.send(Delete).await.unwrap();
        let res = a.send(GetEdges).await.unwrap().unwrap();
        let (id, _) = res.first().unwrap();
        let table = id.table.to_string();
        assert_eq!("e_2", table)
    }
}
