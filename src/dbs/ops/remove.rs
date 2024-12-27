use crate::{dbs::entity::Entity, err::Error, ql::record::Record, resp::Response};
use actix::{Handler, Message};
use std::sync::Arc;

#[derive(Message)]
#[rtype(result = "Result<Response, Error>")]
pub enum Remove {
    Edge(Record),
    Field(String),
}
impl Handler<Remove> for Entity {
    type Result = Result<Response, Error>;

    fn handle(&mut self, msg: Remove, _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            Remove::Edge(id) => {
                if let Entity::Node { edges, .. } = self {
                    edges.remove(&id);
                }
            }
            Remove::Field(field) => match self {
                Entity::Node { fields, .. } => {
                    let field: Arc<str> = field.into();
                    fields.remove(&field);
                }
                Entity::Edge { fields, .. } => {
                    let field: Arc<str> = field.into();
                    fields.remove(&field);
                }
            },
        }
        Ok(Response::Value(self.fields().clone().into()))
    }
}

#[cfg(test)]
mod test {
    use actix::Actor;

    use super::*;
    use crate::{
        dbs::ops::get::Get,
        ql::{
            fields::{Field, Fields},
            value::Value,
        },
    };
    use std::collections::BTreeMap;

    #[actix::test]
    async fn remove_field_test() {
        let node = Entity::new_node(
            Record::new("a", "1"),
            vec![("car".into(), "new".into()), ("speed".into(), 2.into())],
        )
        .start();
        node.send(Remove::Field(String::from("car")))
            .await
            .unwrap()
            .unwrap();
        let result: Value = node
            .send(Get::new(Fields(vec![Field::WildCard]), None))
            .await
            .unwrap()
            .unwrap()
            .try_into()
            .unwrap();
        let correct: BTreeMap<Arc<str>, Value> = BTreeMap::from([
            ("speed".into(), 2.into()),
            ("id".into(), Record::new("a", "1").into()),
        ]);
        assert_eq!(result, correct.into());
    }
}
