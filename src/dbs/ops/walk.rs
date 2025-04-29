use crate::{
    dbs::{entity::Entity, graph::Graph},
    err::Error,
    ql::{condition::Condition, path::Path, record::Record},
    resp::Response,
};
use actix::{Addr, Handler, Message, ResponseFuture};
use reblessive::{tree::Stk, Stack, TreeStack};
use std::{ops::Deref, sync::Arc};

#[derive(Message, PartialEq, Eq)]
#[rtype(result = "Result<Response, Error>")]
pub struct Walk {
    pub path: Arc<Vec<Path>>,
    pub filter: Option<Arc<Condition>>,
    pub idx: usize,
    pub origin: Record,
    // pub graph: Addr<Graph>
}

impl Walk {
    pub fn new(path: Vec<Path>, origin: Record) -> Self {
        let filter = path.first().unwrap().filter.clone();

        Walk {
            path: Arc::new(path),
            origin,
            filter,
            idx: 0,
        }
    }
}

impl Handler<Walk> for Entity {
    type Result = ResponseFuture<Result<Response, Error>>;

    fn handle(
        &mut self,
        Walk {
            path,
            idx,
            filter,
            origin,
        }: Walk,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let entity = self.clone();
        Box::pin(async move {
            let step = path
                .get(idx)
                .ok_or(Error::EdgeIndexExceeded(idx, path.deref().clone()))?;

            if idx == path.len() - 1 {
                let edges = entity
                    .edges()
                    .iter()
                    .filter_map(|(record, edge)| {
                        if !(*record.table == *step.to.0 && record != &origin) {
                            return None;
                        }
                        let Some(edge) = edge.valid_path(&step.dir, &entity) else {
                            return None;
                        };
                        Some(edge.clone())
                    })
                    .collect::<Vec<Addr<Entity>>>();

                return Ok(Response::Nodes(edges));
            }
            if let Some(filter) = filter {
                // TODO have to configure filtering
                let mut stack = TreeStack::new();
                // let x = stack
                //     .enter(|stk| filter.evaluate(stk, &graph, &entity.fields(), None))
                //     .finish()
                //     .await?;
                // let mut stk = TreeStack::new();
                // let check = stk
                //     .run(|stk| filter.evaluate(stk, graph, &entity.fields().clone().into()))
                //     .await?;
                // if !check.is_truthy() {
                //     return Ok(Response::None);
                // }
            }

            let mut edges = Vec::new();

            for (rec, edge) in entity.edges() {
                if !(*rec.table == *step.to.0 && rec != &origin) {
                    continue;
                }

                let Some(edge) = edge.valid_path(&step.dir, &entity) else {
                    continue;
                };

                let filter = path.get(idx).unwrap().filter.clone();
                let walk = Walk {
                    path: path.clone(),
                    idx: idx + 1,
                    filter,
                    origin: entity.id().clone(),
                };
                match edge.send(walk).await.unwrap()? {
                    Response::Nodes(v) => edges.append(&mut v.clone()),
                    Response::None => (),
                    _ => unreachable!(),
                }
            }

            Ok(Response::Nodes(edges))
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        dbs::ops::{get::Get, relate::Relate},
        ql::{direction::Direction, fields::Field, fields::Fields},
    };
    use actix::Actor;

    #[actix::test]
    async fn walk_test() {
        let a_id = Record::new("a", "1");
        let b_id = Record::new("b", "2");
        let c_id = Record::new("c", "2");
        let a = Entity::new_node(a_id.clone(), Vec::new()).start();
        let b = Entity::new_node(b_id.clone(), Vec::new()).start();
        let c = Entity::new_node(c_id.clone(), Vec::new()).start();

        let _ = b
            .send(Relate {
                edge: "e_1".to_string(),
                fields: vec![],
                org_id: a_id.clone(),
                origin: a.clone(),
            })
            .await;
        let _ = c
            .send(Relate {
                edge: "e_2".to_string(),
                fields: vec![],
                org_id: b_id.clone(),
                origin: b.clone(),
            })
            .await;
        let path = vec![
            Path::new(Direction::In, String::from("e_1").into(), None),
            Path::new(Direction::In, String::from("b").into(), None),
            Path::new(Direction::In, String::from("e_2").into(), None),
            Path::new(Direction::In, String::from("c").into(), None),
        ];

        let response = a.send(Walk::new(path, a_id)).await.unwrap().unwrap();
        let Response::Nodes(nodes) = response else {
            panic!()
        };
        let node = nodes.first().unwrap();
        let get = Get::new(Fields(vec![Field::WildCard]), None);
        let response = node.send(get).await.unwrap().unwrap();

        println!("response: {0:#?}", response);
    }
}
