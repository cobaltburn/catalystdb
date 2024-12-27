use crate::ql::{direction::Direction, table::Tables, value::Value};
use core::fmt;

#[derive(Debug, Clone, PartialEq, Hash, Eq, PartialOrd, Ord)]
pub struct Step {
    pub dir: Direction,
    pub to: Tables,
    pub filter: Option<Value>,
}

impl fmt::Display for Step {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(filter) = &self.filter {
            write!(f, "{}({} WHERE {})", self.dir, self.to, filter)
        } else {
            write!(f, "{}{}", self.dir, self.to)
        }
    }
}
