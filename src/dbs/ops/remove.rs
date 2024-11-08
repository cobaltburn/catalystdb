use std::sync::Arc;

use crate::{
    dbs::{edge::Edge, node::Node},
    ql::record::Record,
};
use actix::{Handler, Message};

#[derive(Message)]
#[rtype(result = "()")]
pub enum Remove {
    Edge(Record),
    Field(String),
}

impl Handler<Remove> for Node {
    type Result = ();

    fn handle(&mut self, msg: Remove, _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            Remove::Edge(id) => {
                self.edges.remove(&id);
            }
            Remove::Field(field) => {
                let field: Arc<str> = field.into();
                self.fields.remove(&field);
            }
        }
    }
}

impl Handler<Remove> for Edge {
    type Result = ();

    fn handle(&mut self, msg: Remove, _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            Remove::Edge(_) => unreachable!(),
            Remove::Field(field) => {
                let field: Arc<str> = field.into();
                self.fields.remove(&field);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        dbs::ops::get::Get,
        ql::{fields::Field, value::Value},
    };
    use actix::Actor;
    use std::collections::BTreeMap;

    #[actix::test]
    async fn remove_field_test() {
        let node = Node::new(
            Record::new("a", "1"),
            vec![("car".into(), "new".into()), ("speed".into(), 2.into())],
        )
        .start();
        node.send(Remove::Field(String::from("car"))).await.unwrap();
        let result: BTreeMap<Arc<str>, Value> = node
            .send(Get::new(vec![Field::WildCard], None))
            .await
            .unwrap()
            .try_into()
            .unwrap();
        let correct: BTreeMap<Arc<str>, Value> = BTreeMap::from([
            ("speed".into(), 2.into()),
            ("id".into(), Record::new("a", "1").into()),
        ]);
        assert_eq!(result, correct);
    }
}
