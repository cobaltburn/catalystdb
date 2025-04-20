use crate::ql::{condition::Condition, direction::Direction, table::Table};
use std::{fmt, sync::Arc};

#[derive(Debug, Clone, PartialEq, Hash, Eq, PartialOrd, Ord)]
pub struct Path {
    pub dir: Direction,
    pub to: Table,
    pub filter: Option<Arc<Condition>>,
}

impl Path {
    pub fn new(dir: Direction, to: Table, filter: Option<Condition>) -> Path {
        let filter = filter.map(|cond| Arc::new(cond));
        Path { dir, to, filter }
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(filter) = &self.filter {
            write!(f, "{}({} WHERE {})", self.dir, self.to, filter)
        } else {
            write!(f, "{}{}", self.dir, self.to)
        }
    }
}
