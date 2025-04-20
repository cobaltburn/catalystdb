use crate::ql::{array::Array, value::Value};

impl Value {
    pub fn flatten(self) -> Self {
        match self {
            Value::Array(array) => array
                .into_iter()
                .flat_map(|val| match val {
                    Value::Array(array) => array,
                    _ => Array(vec![val]),
                })
                .collect::<Vec<_>>()
                .into(),
            _ => self,
        }
    }
}
