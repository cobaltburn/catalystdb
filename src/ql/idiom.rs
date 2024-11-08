use crate::{
    dbs::node::Node,
    err::Error,
    ql::{part::Part, value::Value},
};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Hash, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub struct Idioms(pub Vec<Idiom>);

#[derive(Debug, Clone, PartialEq, Hash, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub struct Idiom(pub Vec<Part>);

impl fmt::Display for Idiom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut idiom_str = String::new();
        for part in self.0.iter() {
            idiom_str.push_str(&format!("{part}."));
        }
        idiom_str.pop();
        write!(f, "{idiom_str}")
    }
}
