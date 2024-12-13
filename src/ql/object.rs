use crate::{
    err::Error,
    ql::{ident::Ident, part::Part, value::Value},
};
use std::{collections::BTreeMap, fmt, sync::Arc};

#[derive(Debug, Clone, Default, PartialEq, Hash, Eq, PartialOrd, Ord)]
pub struct Object(pub BTreeMap<Arc<str>, Value>);

impl From<BTreeMap<Arc<str>, Value>> for Object {
    fn from(value: BTreeMap<Arc<str>, Value>) -> Self {
        Object(value)
    }
}

impl Object {
    pub fn get(&self, field: &Arc<str>) -> Option<&Value> {
        self.0.get(field)
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
            Part::Field(Ident(id)) => self
                .get(id)
                .ok_or_else(|| Error::FieldNotFound(id.to_string()))?
                .to_owned(),
            _ => return Err(Error::FieldNotFound(part.to_string())),
        })
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let obj: BTreeMap<&str, &Value> = self.0.iter().map(|(k, v)| (&**k, v)).collect();
        write!(f, "{:#?}", obj)
    }
}
