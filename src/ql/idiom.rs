use crate::{
    dbs::graph::Graph,
    doc::document::Cursor,
    err::Error,
    ql::{ident::Ident, part::Part, value::Value},
};
use actix::Addr;
use reblessive::tree::Stk;
use std::sync::Arc;
use std::{collections::BTreeMap, fmt, ops::Deref, slice::Iter};

use super::traits::Incoperate;

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Hash, Eq, PartialOrd, Ord)]
pub struct Idioms(pub Vec<Idiom>);

impl Deref for Idioms {
    type Target = Vec<Idiom>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl IntoIterator for Idioms {
    type Item = Idiom;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Hash, Eq, PartialOrd, Ord)]
pub struct Idiom(pub Vec<Part>);

impl Idiom {
    pub async fn evaluate(
        &self,
        stk: &mut Stk,
        graph: &Addr<Graph>,
        cur: Option<&Cursor>,
    ) -> Result<Value, Error> {
        match self.first() {
            Some(Part::Start(v)) => stk.run(|stk| v.evaluate(stk, graph, cur)).await,
            _ => match cur {
                Some(_) => todo!(),
                None => todo!(),
            },
        }
    }

    /* pub async fn execute(
        self,
        origin: Record,
        graph: &Addr<Graph>,
        stm: &Statement<'_>,
    ) -> Result<Value, Error> {
        let Idiom(parts) = self;
        let part = parts.first().unwrap();

        match part {
            Part::Step(_) => {
                let mut parts = parts.into_iter().peekable();

                let mut path = Vec::new();
                while let Some(Part::Step(step)) = parts.next() {
                    path.push(step);
                }

                let field: Field = parts
                    .next_if(|e| matches!(e, Part::Field(_)) || matches!(e, Part::All))
                    .map_or(Part::Field(Ident("id".into())).try_into(), |part| {
                        part.try_into()
                    })?;

                let response = graph.send(Retrieve::Record(origin.clone())).await.unwrap();

                let node = match response {
                    Response::Record(addr) => addr,
                    Response::None => return Ok(Value::None),
                    _ => unreachable!(),
                };

                let response = node.send(Walk::new(path, origin, field)).await.unwrap()?;
                let value: Value = response.try_into()?;
                // need to deal with this
            }
            Part::Edge(_) => todo!(),
            Part::Start(_) => todo!(),
            Part::Field(_) => todo!(),
            Part::Index(_) => todo!(),
            Part::Value(_) => todo!(),
            Part::All => todo!(),
            Part::Flatten => todo!(),
            Part::First => todo!(),
            Part::Last => todo!(),
            Part::Where(value) => todo!(),
        }
        todo!()
    } */

    fn parse_walk(start: Part, parts: &mut Iter<Part>) -> Result<Value, Error> {
        // let mut steps = vec![];
        while let Some(part) = parts.next() {}
        todo!()
    }
}

impl Incoperate for Idiom {
    fn incorperate(&self, fields: &BTreeMap<Arc<str>, Value>) -> Idiom {
        match self.split_first().expect("how did you get an empty idiom") {
            (Part::Field(Ident(hd)), tl) => match fields.get(hd) {
                Some(v) => {
                    let mut tl = tl.to_vec();
                    tl.insert(0, Part::Value(v.to_owned()));
                    Idiom(tl)
                }
                None => self.to_owned(),
            },
            _ => self.to_owned(),
        }
    }
}

impl From<Vec<Part>> for Idiom {
    fn from(parts: Vec<Part>) -> Self {
        Idiom(parts)
    }
}

impl Deref for Idiom {
    type Target = Vec<Part>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for Idiom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut idiom_str = String::new();
        for part in self.0.iter() {
            if let Part::Index(part) = part {
                idiom_str.push_str(&format!("[{part}]"));
            } else {
                idiom_str.push_str(&format!("{part}."));
            }
        }
        let idiom_str = idiom_str.trim_end_matches('.');
        write!(f, "{idiom_str}")
    }
}

/* #[cfg(test)]
mod test {
    use std::collections::BTreeMap;

    use super::*;
    use crate::ql::{ident::Ident, object::Object, record::Record};

    #[test]
    fn test_evaluate_all() {
        let idiom = Idiom(Vec::from([Part::All]));
        let object_2 = Value::Object(Object(BTreeMap::from([
            ("id".into(), Record::new("table", 1).into()),
            ("a".into(), 1.into()),
            ("b".into(), 2.into()),
        ])));

        let object = Object(BTreeMap::from([
            ("a".into(), 1.into()),
            ("b".into(), 2.into()),
            ("id".into(), Record::new("table", 1).into()),
        ]));

        let fields = idiom.evaluate(&object_2).unwrap();

        assert_eq!(fields, Value::Object(object));
    }

    #[test]
    fn test_evaluate_ident() {
        let idiom = Idiom(Vec::from([Part::Field(Ident("a".into()))]));
        let object = Value::Object(Object(BTreeMap::from([
            ("a".into(), 1.into()),
            ("id".into(), Record::new("table", 1).into()),
        ])));

        let field = idiom.evaluate(&object).unwrap();

        assert_eq!(field, 1.into());
    }

    #[test]
    fn test_evaluate_object() {
        let idiom = Idiom(Vec::from([
            Part::Field(Ident("a".into())),
            Part::Field(Ident("b".into())),
        ]));
        let object = Object(BTreeMap::from([("b".into(), 2.into())]));
        let object_2 = Value::Object(Object(BTreeMap::from([
            ("a".into(), Value::Object(object)),
            ("id".into(), Record::new("table", 1).into()),
        ])));

        let field = idiom.evaluate(&object_2).unwrap();

        assert_eq!(field, 2.into());
    }
} */
