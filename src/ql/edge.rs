use crate::ql::{direction::Direction, record::Record, table::Table};
use core::fmt;

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub struct Edge {
    pub dir: Direction,
    pub from: Record,
    pub to: Table,
}

impl fmt::Display for Edge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Edge { dir, from, to } = self;
        write!(f, "{from}{dir}{to}")
    }
}
