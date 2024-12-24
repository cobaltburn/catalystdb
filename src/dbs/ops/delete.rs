use crate::{dbs::entity::Entity, err::Error, resp::Response};
use actix::{ActorContext, Handler, Message};

#[derive(Message)]
#[rtype(result = "Result<Response, Error>")]
pub struct Delete;

impl Handler<Delete> for Entity {
    type Result = Result<Response, Error>;

    fn handle(&mut self, _msg: Delete, ctx: &mut Self::Context) -> Self::Result {
        ctx.stop();
        Ok(Response::None)
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use super::*;
    use crate::{dbs::ops::relate::Relate, ql::record::Record};
    use actix::{Actor, Addr};

    #[derive(Message)]
    #[rtype(result = "Option<Vec<(Record, Addr<Entity>)>>")]
    pub struct GetEdges;

    impl Handler<GetEdges> for Entity {
        type Result = Option<Vec<(Record, Addr<Entity>)>>;

        fn handle(&mut self, _msg: GetEdges, _ctx: &mut Self::Context) -> Self::Result {
            let edges = self.edges().clone().into_iter().collect();
            Some(edges)
        }
    }

    #[actix::test]
    async fn delete_test() {
        let fields: Vec<(Arc<str>, _)> = Vec::new();
        let a_id = Record::new("a", "1");
        let b_id = Record::new("b", "2");
        let a = Entity::new_node(a_id, fields.clone()).start();
        let b = Entity::new_node(b_id.clone(), fields.clone()).start();
        let _ = a
            .send(Relate {
                edge: "e_1".to_string(),
                fields: vec![],
                org_id: b_id,
                origin: b.clone(),
            })
            .await;
        b.send(Delete).await.unwrap().unwrap();
        let res = a.send(GetEdges).await.unwrap().unwrap();
        assert!(res.is_empty())
    }

    #[actix::test]
    async fn delete_two_test() {
        let fields: Vec<(Arc<str>, _)> = Vec::new();
        let a_id = Record::new("a", "1");
        let b_id = Record::new("b", "2");
        let c_id = Record::new("c", "2");
        let a = Entity::new_node(a_id, fields.clone()).start();
        let b = Entity::new_node(b_id.clone(), fields.clone()).start();
        let c = Entity::new_node(c_id.clone(), fields.clone()).start();
        let _ = a
            .send(Relate {
                edge: "e_1".to_string(),
                fields: vec![],
                org_id: b_id,
                origin: b.clone(),
            })
            .await;
        let _ = a
            .send(Relate {
                edge: "e_2".to_string(),
                fields: vec![],
                org_id: c_id,
                origin: c.clone(),
            })
            .await;
        b.send(Delete).await.unwrap().unwrap();
        let res = a.send(GetEdges).await.unwrap().unwrap();
        let (id, _) = res.first().unwrap();
        let table = id.table.to_string();
        assert_eq!("e_2", table)
    }
}
