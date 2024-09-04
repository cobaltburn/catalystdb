use crate::{
    graph::{edge::Edge, node::Node},
    ql::{record::Record, value::Value},
};
use actix::{Actor, Addr, AsyncContext, Handler, Message};

#[derive(Message)]
#[rtype(result = "()")]
pub enum Relate {
    Generate {
        edge: String,
        fields: Vec<(String, Value)>,
        from: Record,
        address: Addr<Node>,
    },
    Relate {
        edge: String,
        fields: Vec<(String, Value)>,
        address: Addr<Node>,
    },
}

impl Relate {
    pub fn generate(
        edge: String,
        fields: Vec<(String, Value)>,
        from: Record,
        address: Addr<Node>,
    ) -> Self {
        Relate::Generate {
            edge,
            fields,
            from,
            address,
        }
    }
}

impl Handler<Relate> for Node {
    type Result = ();

    fn handle(&mut self, msg: Relate, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            Relate::Generate {
                edge,
                fields,
                from,
                address,
            } => {
                Edge::new(edge, self.id(), from, ctx.address(), address, fields).start();
            }
            Relate::Relate {
                edge,
                fields,
                address,
            } => address.do_send(Relate::generate(edge, fields, self.id(), ctx.address())),
        };
    }
}
