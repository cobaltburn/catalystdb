use crate::{
    dbs::{node::Node, ops::response::Response},
    ql::{
        fields::{Field, Fields},
        value::Value,
    },
};
use actix::{Handler, Message};
use std::collections::BTreeMap;

#[derive(Message)]
#[rtype(result = "Response")]
pub struct Get {
    pub fields: Fields,
    pub filter: Option<Value>,
}

impl Get {
    pub fn new<V: Into<Fields>>(fields: V, filter: Option<Value>) -> Self {
        Get {
            fields: fields.into(),
            filter,
        }
    }
}

impl Handler<Get> for Node {
    type Result = Response;

    fn handle(&mut self, Get { fields, filter }: Get, _ctx: &mut Self::Context) -> Self::Result {
        if let Some(filter) = filter {
            let check = match filter.evaluate(self) {
                Ok(v) => v,
                Err(e) => return Response::Error(e),
            };

            if !check.is_truthy() {
                return Response::None;
            }
        };

        let mut object = BTreeMap::new();
        for field in fields {
            match field {
                Field::WildCard => object.append(&mut self.fields()),
                Field::Single { expr, alias } => {
                    let key = alias.map_or(expr.to_string().into(), Into::into);
                    let value = self.evaluate(&expr).unwrap_or(Value::None);
                    object.insert(key, value);
                }
            };
        }

        Response::Value(object.into())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        dbs::node::Node,
        ql::{ident::Ident, idiom::Idiom, part::Part, record::Record},
    };
    use actix::Actor;
    use std::sync::Arc;

    #[actix_rt::test]
    async fn get_wildcard_test() {
        let node = Node::new(
            Record::new("a", 1),
            vec![("b".into(), 2.into()), ("c".into(), "c".into())],
        );
        let node = node.start();
        let res = node
            .send(Get::new(Fields::new(vec![Field::WildCard]), None))
            .await
            .unwrap();

        let res: Value = res.try_into().unwrap();
        let expect: BTreeMap<Arc<str>, Value> = BTreeMap::from([
            ("b".into(), 2.into()),
            ("c".into(), "c".into()),
            ("id".into(), Record::new("a", 1).into()),
        ]);

        assert_eq!(res, expect.into())
    }

    #[actix_rt::test]
    async fn get_wildcard_alias_test() {
        let node = Node::new(
            Record::new("a", 1),
            vec![("b".into(), 2.into()), ("c".into(), "c".into())],
        );
        let node = node.start();
        let res = node
            .send(Get::new(
                Fields::new(vec![
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

        let res: Value = res.try_into().unwrap();
        let expect: BTreeMap<Arc<str>, Value> = BTreeMap::from([
            ("b".into(), 2.into()),
            ("c".into(), "c".into()),
            ("alias".into(), "c".into()),
            ("id".into(), Record::new("a", 1).into()),
        ]);

        assert_eq!(res, expect.into())
    }

    #[actix_rt::test]
    async fn get_field_test() {
        let node = Node::new(
            Record::new("a", 1),
            vec![("b".into(), 2.into()), ("c".into(), "c".into())],
        );
        let node = node.start();
        let res = node
            .send(Get::new(
                Fields::new(vec![
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

        let res: Value = res.try_into().unwrap();
        let expect: BTreeMap<Arc<str>, Value> = BTreeMap::from([
            ("c".into(), "c".into()),
            ("id".into(), Record::new("a", 1).into()),
        ]);

        assert_eq!(res, expect.into())
    }

    #[actix_rt::test]
    async fn get_field_alias_test() {
        let node = Node::new(
            Record::new("a", 1),
            vec![("b".into(), 2.into()), ("c".into(), "c".into())],
        );
        let node = node.start();
        let res = node
            .send(Get::new(
                Fields::new(vec![
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

        let res: Value = res.try_into().unwrap();
        let expect: BTreeMap<Arc<str>, Value> = BTreeMap::from([
            ("alias".into(), "c".into()),
            ("id".into(), Record::new("a", 1).into()),
        ]);

        assert_eq!(res, expect.into())
    }
}
