use crate::{
    dbs::{edge::Edge, node::Node, ops::response::Response},
    ql::record::Record,
};
use actix::{Handler, MailboxError, Message, ResponseFuture};
use std::sync::Arc;

#[derive(Message)]
#[rtype(result = "Result<Vec<Response>, MailboxError>")]
pub struct Walk(pub Vec<Arc<str>>, pub Arc<str>);

impl Walk {
    pub fn new<W: Into<Arc<str>>, T: Into<Arc<str>>>(path: Vec<W>, field: T) -> Self {
        let path = path.into_iter().map(Into::into).collect();
        Walk(path, field.into())
    }
}

impl Handler<Walk> for Node {
    type Result = ResponseFuture<Result<Vec<Response>, MailboxError>>;

    fn handle(&mut self, walk: Walk, _ctx: &mut Self::Context) -> Self::Result {
        let Walk(mut path, field) = walk;
        let Some(edge) = path.pop() else {
            let fields = if &*field == "*" {
                self.fields().into()
            } else if &*field == "id" {
                self.id().into()
            } else if let Some(value) = self.fields.get(&field) {
                value.clone().into()
            } else {
                Response::None
            };

            return Box::pin(async move { Ok(vec![fields]) });
        };
        let edges = self
            .edges
            .iter()
            .filter_map(|(Record { table, id: _ }, addr)| {
                if *table == edge {
                    Some(addr.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        Box::pin(async move {
            let mut responses = Vec::new();
            for edge in edges {
                let resp = edge.send(Walk::new(path.clone(), field.clone())).await??;
                responses.push(resp);
            }
            Ok(responses.into_iter().flatten().collect())
        })
    }
}

impl Handler<Walk> for Edge {
    type Result = ResponseFuture<Result<Vec<Response>, MailboxError>>;

    fn handle(&mut self, Walk(mut path, field): Walk, _ctx: &mut Self::Context) -> Self::Result {
        let Some(table) = path.pop() else {
            let fields = if &*field == "*" {
                self.fields().into()
            } else if &*field == "id" {
                self.id().into()
            } else if let Some(value) = self.fields.get(&field) {
                value.clone().into()
            } else {
                Response::None
            };

            return Box::pin(async move { Ok(vec![fields]) });
        };
        if self.dest.id.table != table && &*table != "?" {
            return Box::pin(async move { Ok(Vec::new()) });
        }
        let edge = self.dest.address();
        Box::pin(async move { edge.send(Walk::new(path.clone(), field)).await? })
    }
}

#[cfg(test)]
mod test {}
