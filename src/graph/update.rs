use crate::{graph::node::Node, ql::value::Value};
use actix::{Handler, Message};
use std::sync::Arc;

#[derive(Message)]
#[rtype(result = "()")]
pub struct Update<S: Into<Arc<str>>>(pub Vec<(S, Value)>);

impl<S: Into<Arc<str>>> Handler<Update<S>> for Node {
    type Result = ();

    fn handle(&mut self, Update(fields): Update<S>, _ctx: &mut Self::Context) -> Self::Result {
        for (field, value) in fields {
            let field = field.into();
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
    use super::*;
    use crate::{
        graph::get::{FieldIds, Get},
        ql::record::Record,
    };
    use actix::Actor;

    #[actix::test]
    async fn update_test() {
        let a = Node::new(("a", "1").into(), vec![]).start();
        a.send(Update(vec![("car", "new".into()), ("speed", 2.into())]))
            .await
            .unwrap();
        let res: Vec<(Arc<str>, Value)> = a
            .send(Get(FieldIds::WildCard))
            .await
            .unwrap()
            .try_into()
            .unwrap();
        let correct: Vec<(String, Value)> = vec![
            ("car".into(), "new".into()),
            ("speed".into(), 2.into()),
            ("id".into(), Record::new("a", "1").into()),
        ];
        res.into_iter()
            .zip(correct.into_iter())
            .for_each(|((k1, v1), (k2, v2))| {
                assert_eq!(*k1, k2);
                assert_eq!(v1, v2);
            });
    }
}
