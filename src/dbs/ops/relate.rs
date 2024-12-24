use crate::{
    dbs::entity::Entity,
    err::Error,
    ql::{record::Record, value::Value},
    resp::Response,
};
use actix::{Actor, Addr, AsyncContext, Handler, Message};

#[derive(Message)]
#[rtype(result = "Result<Response, Error>")]
pub struct Relate {
    pub edge: String,
    pub fields: Vec<(String, Value)>,
    pub org_id: Record,
    pub origin: Addr<Entity>,
}

impl Handler<Relate> for Entity {
    type Result = Result<Response, Error>;

    fn handle(
        &mut self,
        Relate {
            edge,
            fields,
            org_id,
            origin,
        }: Relate,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        Entity::new_edge(
            edge,
            self.id().clone(),
            org_id,
            ctx.address(),
            origin,
            fields,
        )
        .start();
        Ok(Response::None)
    }
}
