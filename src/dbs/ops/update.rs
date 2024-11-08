use crate::{dbs::node::Node, ql::value::Value};
use actix::{Handler, Message};
use std::sync::Arc;

#[derive(Message)]
#[rtype(result = "()")]
pub struct Update(pub Vec<(Arc<str>, Value)>);

impl Update {
    pub fn new<S: Into<Arc<str>>>(fields: Vec<(S, Value)>) -> Self {
        let fields = fields
            .into_iter()
            .map(|(k, v)| (k.into(), v))
            .collect::<Vec<(Arc<str>, Value)>>();
        Update(fields)
    }
}

impl Handler<Update> for Node {
    type Result = ();

    fn handle(&mut self, Update(fields): Update, _ctx: &mut Self::Context) -> Self::Result {
        for (field, value) in fields {
            if let Some(val) = self.fields.get_mut(&field) {
                *val = value.into();
            } else {
                self.fields.insert(field, value);
            };
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;

    use super::*;
    use crate::{
        dbs::ops::get::Get,
        ql::{fields::Field, record::Record},
    };
    use actix::Actor;

    #[actix::test]
    async fn update_test() {
        let fields: Vec<(Arc<str>, Value)> = Vec::new();
        let a = Node::new(Record::new("a", "1"), fields).start();
        a.send(Update::new(vec![
            ("car", "new".into()),
            ("speed", 2.into()),
        ]))
        .await
        .unwrap();
        let result: BTreeMap<Arc<str>, Value> = a
            .send(Get::new(vec![Field::WildCard], None))
            .await
            .unwrap()
            .try_into()
            .unwrap();
        let correct: BTreeMap<Arc<str>, Value> = BTreeMap::from([
            ("car".into(), "new".into()),
            ("speed".into(), 2.into()),
            ("id".into(), Record::new("a", "1").into()),
        ]);
        assert_eq!(result, correct);
    }
}
