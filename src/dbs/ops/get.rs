use crate::{
    dbs::entity::Entity,
    err::Error,
    ql::{
        condition::Condition,
        fields::{Field, Fields},
        value::Value,
    },
    resp::Response,
};
use actix::{Handler, Message};
use std::collections::BTreeMap;

#[derive(Message, Clone)]
#[rtype(result = "Result<Response, Error>")]
pub struct Get {
    pub fields: Fields,
    pub filter: Option<Condition>,
}

impl Get {
    pub fn new(fields: Fields, filter: Option<Condition>) -> Self {
        Get { fields, filter }
    }
}

impl Handler<Get> for Entity {
    type Result = Result<Response, Error>;

    fn handle(&mut self, Get { fields, filter }: Get, _ctx: &mut Self::Context) -> Self::Result {
        if let Some(filter) = filter {
            // TODO have to configure filtering
            // let check = filter.evaluate(&self.fields().clone().into())?;

            // if !check.is_truthy() {
            //     return Ok(Response::None);
            // }
        };

        let mut object = BTreeMap::new();
        for field in fields {
            match field {
                Field::WildCard => object.append(&mut self.fields().clone()),
                Field::Single { expr, alias } => {
                    let key = alias.map_or(expr.to_string().into(), Into::into);
                    let value = self.evaluate(&expr).unwrap_or(Value::None);
                    object.insert(key, value);
                }
            };
        }

        Ok(Response::Value(object.into()))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ql::{
        expression::Expression, ident::Ident, idiom::Idiom, object::Object, operator::Operator,
        part::Part, record::Record,
    };
    use actix::Actor;
    use std::sync::Arc;

    #[actix_rt::test]
    async fn get_wildcard_alias_test() {
        let node = Entity::new_node(
            Record::new("a", 1),
            vec![("b".into(), 2.into()), ("c".into(), "c".into())],
        );
        let node = node.start();
        let res = node
            .send(Get::new(
                Fields(vec![
                    Field::WildCard,
                    Field::Single {
                        expr: Value::Idiom(Idiom(vec![Part::Field(Ident("c".into()))])),
                        alias: Some(String::from("alias")),
                    },
                ]),
                None,
            ))
            .await
            .unwrap();

        let left: Value = res.unwrap().try_into().unwrap();
        let right = Value::Object(Object(BTreeMap::from([
            ("b".into(), 2.into()),
            ("c".into(), "c".into()),
            ("alias".into(), "c".into()),
            ("id".into(), Record::new("a", 1).into()),
        ])));

        assert_eq!(left, right)
    }

    #[actix_rt::test]
    async fn get_wildcard_test() {
        let node = Entity::new_node(
            Record::new("a", 1),
            vec![("b".into(), 2.into()), ("c".into(), "c".into())],
        );
        let node = node.start();
        let res = node
            .send(Get::new(Fields(vec![Field::WildCard]), None))
            .await
            .unwrap();

        let res: Value = res.unwrap().try_into().unwrap();
        let expect: BTreeMap<Arc<str>, Value> = BTreeMap::from([
            ("b".into(), 2.into()),
            ("c".into(), "c".into()),
            ("id".into(), Record::new("a", 1).into()),
        ]);

        assert_eq!(res, expect.into())
    }

    #[actix_rt::test]
    async fn get_wildcard_filter_test() {
        let node = Entity::new_node(
            Record::new("a", 1),
            vec![("b".into(), 2.into()), ("c".into(), "c".into())],
        );
        let node = node.start();
        let res = node
            .send(Get::new(
                Fields(vec![Field::WildCard]),
                Some(Condition(Value::Expression(Box::new(Expression::Binary {
                    left: Value::Idiom(Idiom(vec![Part::Field(Ident("c".into()))])),
                    op: Operator::Eq,
                    right: "c".into(),
                })))),
            ))
            .await
            .unwrap();

        let res: Value = res.unwrap().try_into().unwrap();
        let expect: BTreeMap<Arc<str>, Value> = BTreeMap::from([
            ("b".into(), 2.into()),
            ("c".into(), "c".into()),
            ("id".into(), Record::new("a", 1).into()),
        ]);

        assert_eq!(res, expect.into())
    }

    #[actix_rt::test]
    async fn get_wildcard_filter_false_test() {
        let node = Entity::new_node(
            Record::new("a", 1),
            vec![("b".into(), 2.into()), ("c".into(), "c".into())],
        );
        let node = node.start();
        let res = node
            .send(Get::new(
                Fields(vec![Field::WildCard]),
                Some(Condition(Value::Expression(Box::new(Expression::Binary {
                    left: Value::Idiom(Idiom(vec![Part::Field(Ident("b".into()))])),
                    op: Operator::Eq,
                    right: "c".into(),
                })))),
            ))
            .await
            .unwrap();

        let left = res.unwrap();
        let right = Response::None;
        assert_eq!(left, right);
    }

    #[actix_rt::test]
    async fn get_field_test() {
        let node = Entity::new_node(
            Record::new("a", 1),
            vec![("b".into(), 2.into()), ("c".into(), "c".into())],
        );
        let node = node.start();
        let res = node
            .send(Get::new(
                Fields(vec![
                    Field::Single {
                        expr: Value::Idiom(Idiom(vec![Part::Field(Ident("c".into()))])),
                        alias: None,
                    },
                    Field::Single {
                        expr: Value::Idiom(Idiom(vec![Part::Field(Ident("id".into()))])),
                        alias: None,
                    },
                ]),
                None,
            ))
            .await
            .unwrap();

        let res: Value = res.unwrap().try_into().unwrap();
        let expect: BTreeMap<Arc<str>, Value> = BTreeMap::from([
            ("c".into(), "c".into()),
            ("id".into(), Record::new("a", 1).into()),
        ]);

        assert_eq!(res, expect.into())
    }

    #[actix_rt::test]
    async fn get_field_alias_test() {
        let node = Entity::new_node(
            Record::new("a", 1),
            vec![("b".into(), 2.into()), ("c".into(), "c".into())],
        );
        let node = node.start();
        let res = node
            .send(Get::new(
                Fields(vec![
                    Field::Single {
                        expr: Value::Idiom(Idiom(vec![Part::Field(Ident("c".into()))])),
                        alias: Some(String::from("alias")),
                    },
                    Field::Single {
                        expr: Value::Idiom(Idiom(vec![Part::Field(Ident("id".into()))])),
                        alias: None,
                    },
                ]),
                None,
            ))
            .await
            .unwrap();

        let res: Value = res.unwrap().try_into().unwrap();
        let expect: BTreeMap<Arc<str>, Value> = BTreeMap::from([
            ("alias".into(), "c".into()),
            ("id".into(), Record::new("a", 1).into()),
        ]);

        assert_eq!(res, expect.into())
    }
}
