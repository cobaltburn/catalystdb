use crate::{
    dbs::entity::Entity,
    err::Error,
    ql::{array::Array, fields::Field, record::Record, value::Value},
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
    pub table: String,
    pub filter: Option<Value>,
}

impl Step {
    pub fn new(table: String, filter: Option<Value>) -> Step {
        Step { table, filter }
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

            let Step { table, filter } = path.get(idx).unwrap();
            let mut values = Vec::new();

            for (rec, edge) in entity.edges() {
                if !(*rec.table == *table && rec != &origin) {
                    continue;
                }

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
        ql::{
            expression::Expression, ident::Ident, idiom::Idiom, number::Number, operator::Operator,
            part::Part, strand::Strand,
        },
    };
    use actix::Actor;

    #[actix::test]
    async fn walk_test() {
        let fields_1: Vec<(Arc<str>, _)> = Vec::from([("val1".into(), 1.into())]);
        let fields_2: Vec<(Arc<str>, _)> = Vec::from([("val2".into(), 2.into())]);
        let fields_3: Vec<(Arc<str>, _)> = Vec::from([("val3".into(), 3.into())]);
        let a_id = Record::new("a", "1");
        let b_id = Record::new("b", "2");
        let c_id = Record::new("c", "2");
        let a = Entity::new_node(a_id.clone(), fields_1).start();
        let b = Entity::new_node(b_id.clone(), fields_2).start();
        let c = Entity::new_node(c_id.clone(), fields_3).start();
        let _ = a
            .send(Relate {
                edge: "e_1".to_string(),
                fields: vec![],
                org_id: b_id,
                origin: b.clone(),
            })
            .await;
        let _ = b
            .send(Relate {
                edge: "e_2".to_string(),
                fields: vec![],
                org_id: c_id.clone(),
                origin: c.clone(),
            })
            .await;
        let path = vec![
            Step::new(String::from("e_1"), None),
            Step::new(String::from("b"), None),
            Step::new(String::from("e_2"), None),
            Step::new(String::from("c"), None),
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
        let _ = a
            .send(Relate {
                edge: "e_1".to_string(),
                fields: vec![],
                org_id: b_id,
                origin: b.clone(),
            })
            .await;

        let _ = b
            .send(Relate {
                edge: "e_2".to_string(),
                fields: vec![],
                org_id: c_id.clone(),
                origin: c.clone(),
            })
            .await;

        let path = vec![
            Step::new(String::from("e_1"), None),
            Step::new(
                String::from("b"),
                Some(Value::Expression(Box::new(Expression::Binary {
                    left: Value::Idiom(Idiom(vec![Part::Field(Ident("val2".into()))])),
                    op: Operator::Eq,
                    right: Value::Number(Number::Int(2)),
                }))),
            ),
            Step::new(String::from("e_2"), None),
            Step::new(
                String::from("c"),
                Some(Value::Expression(Box::new(Expression::Binary {
                    left: Value::Idiom(Idiom(vec![Part::Field(Ident("val3".into()))])),
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
        let _ = a
            .send(Relate {
                edge: "e_1".to_string(),
                fields: vec![],
                org_id: b_id,
                origin: b.clone(),
            })
            .await;

        let _ = b
            .send(Relate {
                edge: "e_2".to_string(),
                fields: vec![],
                org_id: c_id.clone(),
                origin: c.clone(),
            })
            .await;

        let _ = b
            .send(Relate {
                edge: "e_3".to_string(),
                fields: vec![],
                org_id: d_id.clone(),
                origin: d.clone(),
            })
            .await;

        let path = vec![
            Step::new(String::from("e_1"), None),
            Step::new(
                String::from("b"),
                Some(Value::Expression(Box::new(Expression::Binary {
                    left: Value::Idiom(Idiom(vec![Part::Field(Ident("val2".into()))])),
                    op: Operator::Eq,
                    right: Value::Number(Number::Int(2)),
                }))),
            ),
            Step::new(String::from("e_2"), None),
            Step::new(
                String::from("c"),
                Some(Value::Expression(Box::new(Expression::Binary {
                    left: Value::Idiom(Idiom(vec![Part::Field(Ident("x".into()))])),
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
