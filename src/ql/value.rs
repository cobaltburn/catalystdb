use crate::ql::{number::Number, record::Record};
use std::{collections::BTreeMap, sync::Arc};

#[derive(Debug, Clone, Default, PartialEq)]
pub enum Value {
    Record(Record),
    Number(Number),
    String(Arc<str>),
    Bool(bool),
    Array(Vec<Value>),
    Object(BTreeMap<Arc<str>, Value>),
    #[default]
    Null,
}

impl From<serde_json::Value> for Value {
    fn from(value: serde_json::Value) -> Self {
        match value {
            serde_json::Value::Null => Value::Null,
            serde_json::Value::Bool(bool) => Value::Bool(bool),
            serde_json::Value::Number(num) => Value::from(num),
            serde_json::Value::String(st) => Value::String(st.into()),
            serde_json::Value::Array(vals) => {
                Value::Array(vals.into_iter().map(Into::into).collect())
            }
            serde_json::Value::Object(obj) => {
                Value::Object(obj.into_iter().map(|(k, v)| (k.into(), v.into())).collect())
            }
        }
    }
}

impl From<serde_json::Number> for Value {
    fn from(value: serde_json::Number) -> Self {
        if let Some(num) = value.as_i64() {
            Value::Number(Number::Int(num))
        } else if let Some(num) = value.as_u64() {
            Value::Number(Number::Int(num as i64))
        } else {
            Value::Number(Number::Float(value.as_f64().unwrap()))
        }
    }
}

impl<T> From<Option<T>> for Value
where
    Value: From<T>,
{
    fn from(value: Option<T>) -> Self {
        match value {
            Some(value) => Value::from(value),
            None => Value::Null,
        }
    }
}

impl From<Record> for Value {
    fn from(value: Record) -> Self {
        Value::Record(value)
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::String(value.into())
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Value::String(value.into())
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Bool(value)
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Value::Number(value.into())
    }
}
