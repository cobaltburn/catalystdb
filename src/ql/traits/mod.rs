use crate::ql::value::Value;
use std::{collections::BTreeMap, sync::Arc};

pub trait Incoperate {
    fn incorperate(&self, fields: &BTreeMap<Arc<str>, Value>) -> Self;
}
