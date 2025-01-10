use crate::ql::value::Value;
use std::sync::Arc;

impl Value {
    pub fn get(&self, key: &Arc<str>) -> Value {
        match self {
            Value::Object(object) => object.get(key).clone(),
            Value::Array(array) => array
                .iter()
                .map(|val| val.get(key))
                .collect::<Vec<_>>()
                .into(),
            _ => Value::None,
        }
    }
}
