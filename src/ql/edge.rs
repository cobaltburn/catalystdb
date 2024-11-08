use crate::ql::{record::Record, table::Table};

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct Edge {
    pub origin: Record,
    pub dest: Table,
}

impl Edge {
    pub fn new(origin: Record, dest: Table) -> Self {
        Edge { origin, dest }
    }
}
