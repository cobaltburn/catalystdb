use actix::Addr;
use reblessive::tree::{self, Stk};

use crate::{
    dbs::graph::Graph,
    doc::document::Cursor,
    err::Error,
    ql::{ident::Ident, part::Part, value::Value},
};
use std::{collections::BTreeMap, fmt, ops::Deref, sync::Arc};

#[derive(Debug, Clone, Default, PartialEq, Hash, Eq, PartialOrd, Ord)]
pub struct Object(pub BTreeMap<Arc<str>, Value>);

impl From<BTreeMap<Arc<str>, Value>> for Object {
    fn from(value: BTreeMap<Arc<str>, Value>) -> Self {
        Object(value)
    }
}

impl From<BTreeMap<String, Value>> for Object {
    fn from(v: BTreeMap<String, Value>) -> Self {
        let v = v.into_iter().map(|(s, v)| (s.into(), v)).collect();
        Object(v)
    }
}

impl Object {
    pub fn get(&self, field: &Arc<str>) -> &Value {
        self.0.get(field).unwrap_or(&Value::None)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn fields(&self) -> BTreeMap<Arc<str>, Value> {
        self.0.clone()
    }

    pub fn retrieve(&self, part: &Part) -> Result<Value, Error> {
        Ok(match part {
            Part::All => Value::Object(self.fields().into()),
            Part::Field(Ident(id)) => self.get(id).to_owned(),
            _ => return Err(Error::FieldNotFound(part.to_string())),
        })
    }
}

impl Deref for Object {
    type Target = BTreeMap<Arc<str>, Value>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl IntoIterator for Object {
    type Item = (Arc<str>, Value);
    type IntoIter = std::collections::btree_map::IntoIter<Arc<str>, Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let obj: BTreeMap<&str, &Value> = self.0.iter().map(|(k, v)| (&**k, v)).collect();
        write!(f, "{:#?}", obj)
    }
}

impl Object {
    pub async fn evaluate(
        &self,
        stk: &mut Stk,
        graph: &Addr<Graph>,
        cur: Option<&Cursor>,
    ) -> Result<Value, Error> {
        let mut object = BTreeMap::new();
        for (key, v) in self.iter() {
            let val = stk.run(|stk| v.evaluate(stk, graph, cur)).await?;
            object.insert(key.clone(), val);
        }
        Ok(Value::Object(Object(object)))
    }
}
