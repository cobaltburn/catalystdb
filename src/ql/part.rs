use crate::{
    dbs::ops::get::Get,
    ql::{edge::Edge, fields::Field, ident::Ident, number::Number, path::Path, value::Value},
};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Hash, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Part {
    All,
    Flatten,
    First,
    Last,
    Field(Ident),
    Index(Number),
    Where(Value),
    Path(Path),
    Edge(Edge),
    Start(Value),
    Value(Value),
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

pub trait ParseWalk<'a> {
    fn parse_walk(&'a self) -> (Get, Vec<Path>, &'a [Part]);
}

impl<'a> ParseWalk<'a> for &'a [Part] {
    fn parse_walk(&'a self) -> (Get, Vec<Path>, &'a [Part]) {
        let walk_path = self
            .iter()
            .take_while(|&p| Part::is_path(p))
            .filter_map(|p| {
                if let Part::Path(p) = p {
                    Some(p.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<Path>>();

        let (get, path) = if let Some(part @ Part::Field(_)) = self.get(walk_path.len()) {
            let field: Field = part.clone().try_into().unwrap();
            (Get::new(field.into(), None), &self[walk_path.len() + 1..])
        } else {
            let field: Field = Part::Field("id".into()).try_into().unwrap();
            (Get::new(field.into(), None), &self[walk_path.len()..])
        };
        (get, walk_path, path)
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

    pub fn is_path(&self) -> bool {
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

#[cfg(test)]
mod test {}
