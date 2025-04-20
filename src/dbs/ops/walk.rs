use crate::{
    dbs::{entity::Entity, graph::Graph, ops::get::Get},
    err::Error,
    ql::{condition::Condition, fields::Field, path::Path, record::Record, value::Value},
    resp::Response,
};
use actix::{Addr, Handler, Message, ResponseFuture};
use reblessive::{tree::Stk, Stack, TreeStack};
use std::sync::Arc;

#[derive(Message, PartialEq, Eq)]
#[rtype(result = "Result<Response, Error>")]
pub struct Walk {
    pub path: Arc<Vec<Path>>,
    pub filter: Option<Arc<Condition>>,
    pub idx: usize,
    pub origin: Record,
    pub field: Arc<Field>,
    pub graph: Addr<Graph>,
}

impl Walk {
    pub fn new(path: Vec<Path>, origin: Record, field: Field, graph: Addr<Graph>) -> Self {
        let filter = path.first().unwrap().filter.clone();

        Walk {
            path: Arc::new(path),
            origin,
            filter,
            idx: 0,
            field: Arc::new(field),
            graph,
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
            field,
            graph,
        }: Walk,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let entity = self.clone();
        Box::pin(async move {
            let step = path
                .get(idx)
                .ok_or(Error::EdgeIndexExceeded(idx, (*path).clone()))?;

            if idx == path.len() - 1 {
                let mut values = Vec::new();
                for (rec, edge) in entity.edges() {
                    if !(*rec.table == *step.to.0 && rec != &origin) {
                        continue;
                    }

                    let Some(edge) = edge.valid_path(&step.dir, &entity) else {
                        continue;
                    };

                    let get = Get {
                        fields: (*field).clone().into(),
                        filter: filter.clone(),
                    };
                    let response = edge.send(get).await.unwrap();

                    match response {
                        Ok(Response::Value(v)) => values.push(v),
                        Ok(Response::None) => (),
                        Err(_) => return response,
                        _ => unreachable!(),
                    };
                }

                return Ok(Response::Value(values.into()));
            }
            if let Some(filter) = filter {
                let mut stack = TreeStack::new();
                let x = stack
                    .enter(|stk| filter.evaluate(stk, &graph, &entity.fields(), None))
                    .finish()
                    .await?;
                /* let mut stk = Stack::new();
                let check = stk
                    .run(|stk| filter.evaluate(stk, graph, &entity.fields().clone().into()))
                    .await?;
                if !check.is_truthy() {
                    return Ok(Response::None);
                } */
            }

            let mut values = Vec::new();

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
                    field: field.clone(),
                    graph: graph.clone(),
                };

                let response = edge.send(walk).await.unwrap();

                match response {
                    Ok(Response::Value(Value::Array(mut array))) => values.append(&mut array),
                    Ok(Response::None) => (),
                    Err(_) => return response,
                    _ => unreachable!(),
                }
            }

            Ok(Response::Value(values.into()))
        })
    }
}

/* #[cfg(test)]
mod test {
    use super::*;
    use crate::{
        dbs::ops::relate::Relate,
        ql::{
            array::Array, direction::Direction, expression::Expression, ident::Ident,
            operator::Operator, strand::Strand,
        },
    };
    use actix::Actor;

    #[actix::test]
    async fn walk_test() {
        let fields_1 = Vec::new();
        let fields_2 = Vec::new();
        let fields_3 = Vec::new();
        let a_id = Record::new("a", "1");
        let b_id = Record::new("b", "2");
        let c_id = Record::new("c", "2");
        let a = Entity::new_node(a_id.clone(), fields_1).start();
        let b = Entity::new_node(b_id.clone(), fields_2).start();
        let c = Entity::new_node(c_id.clone(), fields_3).start();

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
            Step::new(Direction::In, String::from("e_1").into(), None),
            Step::new(Direction::In, String::from("b").into(), None),
            Step::new(Direction::In, String::from("e_2").into(), None),
            Step::new(Direction::In, String::from("c").into(), None),
        ];

        let response = a
            .send(Walk::new(path, a_id, Field::WildCard))
            .await
            .unwrap()
            .unwrap();
        let value: Value = response.try_into().unwrap();
        let Value::Array(Array(vec)) = value else {
            panic!()
        };
        let value = vec.first().unwrap();
        /* let left = value.get(&"id".into());
        let right = Value::Record(Box::new(c_id.clone()));

        assert_eq!(left, right); */
    }

    #[actix::test]
    async fn walk_filter_test() {
        let a_id = Record::new("a", "1");
        let b_id = Record::new("b", "2");
        let c_id = Record::new("c", "2");
        let a = Entity::new_node(a_id.clone(), vec![("val1".into(), 1.into())]).start();
        let b = Entity::new_node(b_id.clone(), vec![("val2".into(), 2.into())]).start();
        let c = Entity::new_node(c_id.clone(), vec![("val3".into(), "x".into())]).start();
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
            Step::new(Direction::In, String::from("e_1").into(), None),
            Step::new(
                Direction::In,
                String::from("b").into(),
                Some(
                    Value::Expression(Box::new(Expression::Binary {
                        left: Ident("val2".into()).into(),
                        op: Operator::Eq,
                        right: 2.into(),
                    }))
                    .into(),
                ),
            ),
            Step::new(Direction::In, String::from("e_2").into(), None),
            Step::new(
                Direction::In,
                String::from("c").into(),
                Some(
                    Value::Expression(Box::new(Expression::Binary {
                        left: Ident("val3".into()).into(),
                        op: Operator::Eq,
                        right: Value::String(Strand("x".into())),
                    }))
                    .into(),
                ),
            ),
        ];

        let response = a
            .send(Walk::new(path, a_id, Field::WildCard))
            .await
            .unwrap()
            .unwrap();
        let value: Value = response.try_into().unwrap();
        let Value::Array(Array(vec)) = value else {
            panic!()
        };
        let value = vec.first().unwrap();
        let left = value.get(&"id".into());
        let right = Value::Record(Box::new(c_id.clone()));

        assert_eq!(left, right);
    }

    #[actix::test]
    async fn walk_filter_two_path_test() {
        let fields_3: Vec<(Arc<str>, _)> = Vec::from([
            ("x".into(), "x".into()),
            ("y".into(), "y".into()),
            ("z".into(), "z".into()),
        ]);
        let a_id = Record::new("a", "1");
        let b_id = Record::new("b", "2");
        let c_id = Record::new("c", "2");
        let d_id = Record::new("d", "2");
        let a = Entity::new_node(a_id.clone(), vec![("val1".into(), 1.into())]).start();
        let b = Entity::new_node(b_id.clone(), vec![("val2".into(), 2.into())]).start();
        let c = Entity::new_node(c_id.clone(), fields_3).start();
        let d = Entity::new_node(d_id.clone(), vec![("val4".into(), 4.into())]).start();
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

        let _ = d
            .send(Relate {
                edge: "e_3".to_string(),
                fields: vec![],
                org_id: b_id.clone(),
                origin: b.clone(),
            })
            .await;

        let path = vec![
            Step::new(Direction::In, String::from("e_1").into(), None),
            Step::new(
                Direction::In,
                String::from("b").into(),
                Some(
                    Value::Expression(Box::new(Expression::Binary {
                        left: Ident("val2".into()).into(),
                        op: Operator::Eq,
                        right: 2.into(),
                    }))
                    .into(),
                ),
            ),
            Step::new(Direction::In, String::from("e_2").into(), None),
            Step::new(
                Direction::In,
                String::from("c").into(),
                Some(
                    Value::Expression(Box::new(Expression::Binary {
                        left: Ident("x".into()).into(),
                        op: Operator::Eq,
                        right: Value::String(Strand("x".into())),
                    }))
                    .into(),
                ),
            ),
        ];

        let response = a
            .send(Walk::new(path, a_id, Field::WildCard))
            .await
            .unwrap()
            .unwrap();
        let value: Value = response.try_into().unwrap();
        let Value::Array(Array(vec)) = value else {
            panic!()
        };
        let value = vec.first().unwrap();
        let left = value.get(&"id".into());
        let right = Value::Record(Box::new(c_id.clone()));

        assert_eq!(vec.len(), 1);
        assert_eq!(left, right);
    }
} */
