use crate::{graph::node::Node, ql::record::Record};
use actix::{Handler, Message};
use std::sync::Arc;

use super::edge::Edge;

#[derive(Message)]
#[rtype(result = "()")]
pub enum Remove<S: Into<Arc<str>>> {
    Edge(Record),
    Field(S),
}

impl<T: Into<Arc<str>>> Handler<Remove<T>> for Node {
    type Result = ();

    fn handle(&mut self, msg: Remove<T>, _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            Remove::Edge(id) => {
                self.edges.remove(&id);
            }
            Remove::Field(field) => {
                self.fields.remove(&field.into());
            }
        }
    }
}

impl<T: Into<Arc<str>>> Handler<Remove<T>> for Edge {
    type Result = ();

    fn handle(&mut self, msg: Remove<T>, _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            Remove::Edge(_) => unreachable!(),
            Remove::Field(field) => {
                self.fields.remove(&field.into());
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        graph::get::{FieldIds, Get},
        ql::value::Value,
    };
    use actix::Actor;

    #[actix::test]
    async fn remove_field_test() {
        let id = Record::new("a", "1");
        let a = Node::new(
            id,
            vec![("car".into(), "new".into()), ("speed".into(), 2.into())],
        )
        .start();
        a.send(Remove::Field("car")).await.unwrap();
        let res: Vec<(Arc<str>, Value)> = a
            .send(Get(FieldIds::WildCard))
            .await
            .unwrap()
            .try_into()
            .unwrap();
        let correct: Vec<(String, Value)> = vec![
            ("speed".into(), 2.into()),
            ("id".into(), Record::new("a", "1").into()),
        ];
        for ((k1, v1), (k2, v2)) in res.into_iter().zip(correct.into_iter()) {
            assert_eq!(*k1, k2);
            assert_eq!(v1, v2);
        }
    }
}
