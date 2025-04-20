use crate::{dbs::entity::Entity, err::Error, ql::value::Value, resp::Response};
use actix::{Handler, Message};
use std::sync::Arc;

#[derive(Message)]
#[rtype(result = "Result<Response, Error>")]
pub struct Update(pub Vec<(Arc<str>, Value)>);

impl Update {
    pub fn new<T: Into<Arc<str>>>(fields: Vec<(T, Value)>) -> Update {
        let fields = fields.into_iter().map(|(e, v)| (e.into(), v)).collect();
        Update(fields)
    }
}

impl Handler<Update> for Entity {
    type Result = Result<Response, Error>;

    fn handle(&mut self, Update(fields): Update, _ctx: &mut Self::Context) -> Self::Result {
        for (field, value) in fields {
            let fields = match self {
                Entity::Node { fields, .. } => fields,
                Entity::Edge { fields, .. } => fields,
            };
            if let Some(val) = fields.get_mut(&field) {
                *val = value.into();
            } else {
                fields.insert(field, value);
            };
        }
        Ok(Response::Value(self.fields().clone().into()))
    }
}

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;

    use super::*;
    use crate::{
        dbs::ops::get::Get,
        ql::{
            fields::{Field, Fields},
            record::Record,
        },
    };
    use actix::Actor;

    #[actix::test]
    async fn update_test() {
        let fields: Vec<(Arc<str>, Value)> = Vec::new();
        let a = Entity::new_node(Record::new("a", "1"), fields).start();
        a.send(Update::new(vec![
            ("car", "new".into()),
            ("speed", 2.into()),
        ]))
        .await
        .unwrap()
        .unwrap();
        let result: Value = a
            .send(Get::new(Fields(vec![Field::WildCard]), None))
            .await
            .unwrap()
            .unwrap()
            .try_into()
            .unwrap();
        let correct: BTreeMap<Arc<str>, Value> = BTreeMap::from([
            ("car".into(), "new".into()),
            ("speed".into(), 2.into()),
            ("id".into(), Record::new("a", "1").into()),
        ]);
        assert_eq!(result, correct.into());
    }
}
