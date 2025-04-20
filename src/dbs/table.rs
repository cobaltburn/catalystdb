use crate::{
    dbs::entity::Entity,
    ql::{record::Record, value::Value},
    resp::Response,
};
use actix::{Actor, Addr, Context, Handler, Message, ResponseFuture};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

#[derive(Debug)]
pub struct Table {
    pub name: Arc<str>,
    pub nodes: Arc<RwLock<HashMap<Value, Addr<Entity>>>>,
}

impl Actor for Table {
    type Context = Context<Self>;
}

impl Table {
    pub fn new<S: Into<Arc<str>>>(name: S) -> Self {
        Table {
            name: name.into(),
            nodes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn insert(&mut self, value: Value, node: Addr<Entity>) {
        let mut nodes = self.nodes.write().unwrap();
        nodes.insert(value, node);
    }

    pub fn contains(&self, value: &Value) -> bool {
        let nodes = self.nodes.read().unwrap();
        nodes.contains_key(value)
    }
}

#[cfg(test)]
mod test {}
