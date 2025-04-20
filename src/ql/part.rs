use crate::ql::{edge::Edge, ident::Ident, number::Number, path::Path, value::Value};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Hash, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Part {
    All,
    Flatten,
    First,
    Last,
    Start(Value),
    Field(Ident),
    Index(Number),
    Value(Value),
    Where(Value),
    Path(Path),
    Edge(Edge),
}

pub trait Next<'a> {
    fn next(&'a self) -> &'a [Part];
}

impl<'a> Next<'a> for &'a [Part] {
    fn next(&'a self) -> &'a [Part] {
        match self.len() {
            0 => &[],
            _ => &self[1..],
        }
    }
}

pub trait Skip<'a> {
    fn skip(&'a self, amount: usize) -> &'a [Part];
}

impl<'a> Skip<'a> for &'a [Part] {
    fn skip(&'a self, amount: usize) -> &'a [Part] {
        match self.len() {
            0 => &[],
            _ => &self[amount..],
        }
    }
}

impl Part {
    pub fn is_field(&self) -> bool {
        matches!(self, Part::Field(_))
    }

    pub fn is_index(&self) -> bool {
        matches!(self, Part::Index(_))
    }

    pub fn is_value(&self) -> bool {
        matches!(self, Part::Value(_))
    }

    pub fn is_edge(&self) -> bool {
        matches!(self, Part::Edge(_))
    }

    pub fn is_step(&self) -> bool {
        matches!(self, Part::Path(_))
    }

    pub fn is_start(&self) -> bool {
        matches!(self, Part::Start(_))
    }

    pub fn is_where(&self) -> bool {
        matches!(self, Part::Where(_))
    }
}

impl fmt::Display for Part {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Part::All => write!(f, "*"),
            Part::Flatten => todo!(),
            Part::First => write!(f, "[0]"),
            Part::Last => write!(f, "[$]"),
            Part::Field(v) => write!(f, ".{v}"),
            Part::Index(v) => write!(f, "[{v}]"),
            Part::Value(v) => write!(f, "{v}"),
            Part::Edge(v) => write!(f, "[{v}]"),
            Part::Path(v) => write!(f, "{v}"),
            Part::Start(v) => write!(f, "{v}"),
            Part::Where(v) => write!(f, "[WHERE {v}]"),
        }
    }
}
