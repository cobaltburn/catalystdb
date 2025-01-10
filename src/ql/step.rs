use crate::ql::{condition::Condition, direction::Direction, table::Tables};
use core::fmt;

#[derive(Debug, Clone, PartialEq, Hash, Eq, PartialOrd, Ord)]
pub struct Step {
    pub dir: Direction,
    pub to: Tables,
    pub filter: Option<Condition>,
    pub alias: Option<String>,
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
