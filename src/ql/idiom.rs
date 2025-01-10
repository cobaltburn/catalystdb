use crate::{
    err::Error,
    ql::{edge::Edge, part::Part, step::Step, value::Value},
};
use std::{fmt, ops::Deref, slice::Iter};

#[derive(Debug, Clone, PartialEq, Hash, Eq, PartialOrd, Ord)]
#[non_exhaustive]
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

#[derive(Debug, Clone, PartialEq, Hash, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub struct Idiom(pub Vec<Part>);

impl Idiom {
    pub fn evaluate(&self, value: &Value) -> Result<Value, Error> {
        let mut parts = self.iter();
        let part = parts.next().expect("An empty idiom was passed evaluated");
        let mut val = part.evaluate(value)?;

        while let Some(part) = parts.next() {
            val = match part {
                Part::Value(_value) => todo!(),
                Part::Step(_) => Self::parse_walk(part, &mut parts)?,
                Part::Edge(_) => todo!(),
                _ => val.retrieve(part)?,
            }
        }

        Ok(val)
    }

    fn parse_walk(start: &Part, parts: &mut Iter<Part>) -> Result<Value, Error> {
        match start {
            Part::Step(Step {
                dir,
                to,
                filter,
                alias,
            }) => todo!(),
            Part::Edge(Edge { dir, from, to }) => todo!(),
            _ => unreachable!(),
        }
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

#[cfg(test)]
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
}
