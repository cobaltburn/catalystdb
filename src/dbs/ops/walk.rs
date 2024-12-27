use crate::{
    dbs::entity::Entity,
    err::Error,
    ql::{
        array::Array, direction::Direction, fields::Field, record::Record, table::Table,
        value::Value,
    },
    resp::Response,
};
use actix::{Handler, Message, ResponseFuture};
use std::sync::Arc;

#[derive(Message, Debug)]
#[rtype(result = "Result<Response, Error>")]
pub struct Walk {
    pub path: Arc<Vec<Step>>,
    pub idx: usize,
    pub origin: Record,
    pub field: Arc<Field>,
}

impl Walk {
    pub fn new(path: Vec<Step>, origin: Record, field: Arc<Field>) -> Self {
        let path = Arc::new(path);
        let idx = 0;
        Walk {
            path,
            origin,
            idx,
            field,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Step {
    pub dir: Direction,
    pub table: Table,
    pub filter: Option<Value>,
}

impl Step {
    pub fn new(dir: Direction, table: Table, filter: Option<Value>) -> Step {
        Step { dir, table, filter }
    }
}

impl Handler<Walk> for Entity {
    type Result = ResponseFuture<Result<Response, Error>>;

    // TODO deal with selecting singler fields
    fn handle(
        &mut self,
        Walk {
            path,
            idx,
            origin,
            field,
        }: Walk,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let entity = self.clone();
        Box::pin(async move {
            if idx == path.len() {
                let fields = match entity {
                    Entity::Node { fields, .. } => fields,
                    Entity::Edge { fields, .. } => fields,
                }
                .into();
                return Ok(Response::Value(fields));
            }

            let Step { dir, table, filter } = path.get(idx).unwrap();

            let mut values = Vec::new();

            for (rec, edge) in entity.edges() {
                if !(*rec.table == *table.0 && rec != &origin) {
                    continue;
                }

                let edge = match dir {
                    Direction::In if entity.is_edge() && edge.is_out() => edge.edge(),
                    Direction::Out if entity.is_edge() && edge.is_in() => edge.edge(),
                    Direction::In if entity.is_node() && edge.is_in() => edge.edge(),
                    Direction::Out if entity.is_node() && edge.is_out() => edge.edge(),
                    Direction::Both => edge.edge(),
                    _ => continue,
                };

                let walk = Walk {
                    path: path.clone(),
                    idx: idx + 1,
                    origin: entity.id().clone(),
                    field: field.clone(),
                };

                let resp = edge.send(walk).await.unwrap();
                // TODO think of a better way of orginizing the filter this will be slow
                match resp {
                    Ok(Response::Value(obj @ Value::Object(_))) if apply_filter(&obj, filter)? => {
                        values.push(obj)
                    }
                    Ok(Response::Value(Value::Array(Array(mut vec)))) => values.append(&mut vec),
                    Err(Error::None) => (),
                    err @ Err(_) => return err,
                    _ => unreachable!(),
                }
            }
            return Ok(Response::Value(values.into()));
        })
    }
}

fn apply_filter(object: &Value, filter: &Option<Value>) -> Result<bool, Error> {
    if let Some(filter) = filter {
        let check = filter.evaluate(object)?;
        if !check.is_truthy() {
            return Err(Error::None);
        }
    };
    Ok(true)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        dbs::ops::relate::Relate,
        ql::{expression::Expression, ident::Ident, operator::Operator, strand::Strand},
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
            .send(Walk::new(path, a_id, Arc::new(Field::WildCard)))
            .await
            .unwrap()
            .unwrap();
        let value: Value = response.try_into().unwrap();
        let Value::Array(Array(vec)) = value else {
            panic!()
        };
        let value = vec.first().unwrap();
        let left = value.get(&"id".into()).unwrap();
        let right = Value::Record(Box::new(c_id.clone()));
        assert_eq!(left, right);
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
                Some(Value::Expression(Box::new(Expression::Binary {
                    left: Ident("val2".into()).into(),
                    op: Operator::Eq,
                    right: 2.into(),
                }))),
            ),
            Step::new(Direction::In, String::from("e_2").into(), None),
            Step::new(
                Direction::In,
                String::from("c").into(),
                Some(Value::Expression(Box::new(Expression::Binary {
                    left: Ident("val3".into()).into(),
                    op: Operator::Eq,
                    right: Value::String(Strand("x".into())),
                }))),
            ),
        ];

        let response = a
            .send(Walk::new(path, a_id, Arc::new(Field::WildCard)))
            .await
            .unwrap()
            .unwrap();
        let value: Value = response.try_into().unwrap();
        let Value::Array(Array(vec)) = value else {
            panic!()
        };
        let value = vec.first().unwrap();
        let left = value.get(&"id".into()).unwrap();
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
                Some(Value::Expression(Box::new(Expression::Binary {
                    left: Ident("val2".into()).into(),
                    op: Operator::Eq,
                    right: 2.into(),
                }))),
            ),
            Step::new(Direction::In, String::from("e_2").into(), None),
            Step::new(
                Direction::In,
                String::from("c").into(),
                Some(Value::Expression(Box::new(Expression::Binary {
                    left: Ident("x".into()).into(),
                    op: Operator::Eq,
                    right: Value::String(Strand("x".into())),
                }))),
            ),
        ];

        let response = a
            .send(Walk::new(path, a_id, Arc::new(Field::WildCard)))
            .await
            .unwrap()
            .unwrap();
        let value: Value = response.try_into().unwrap();
        let Value::Array(Array(vec)) = value else {
            panic!()
        };
        let value = vec.first().unwrap();
        let left = value.get(&"id".into()).unwrap();
        let right = Value::Record(Box::new(c_id.clone()));
        assert_eq!(left, right);
    }
}
