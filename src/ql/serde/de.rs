/* use crate::ql::value::Value;
use crate::{err::Error, ql::array::Array};
use serde::de::DeserializeOwned;
use serde_content::{Deserializer, Number, Serializer, Value as Content};
use std::borrow::Cow; */

/* impl Value {
    fn into_content(self) -> Result<Content<'static>, Error> {
        let serializer = Serializer::new();
        match self {
            Value::None => Ok(Content::Option(None)),
            Value::Null => Ok(Content::Option(None)),
            Value::Record(_) => todo!(),
            Value::Uuid(v) => serializer.serialize(v).map_err(Into::into),
            Value::Number(v) => match v {
                crate::ql::number::Number::Int(v) => Ok(Content::Number(Number::I64(v))),
                crate::ql::number::Number::Float(v) => Ok(Content::Number(Number::F64(v))),
            },
            Value::String(v) => Ok(Content::String(Cow::Owned(v.to_string()))),
            Value::Bool(v) => Ok(Content::Bool(v)),
            Value::Array(Array(v)) => {
                let mut vec = Vec::with_capacity(v.len());
                for val in v {
                    vec.push(val.into_content()?);
                }
                Ok(Content::Seq(vec))
            }
            Value::Object(_) => todo!(),
            Value::Idiom(v) => serializer.serialize(v).map_err(Into::into),
        }
    }
}

pub fn from_value<T>(value: Value) -> Result<T, Error>
where
    T: DeserializeOwned,
{
    let content = value.into_content()?;
    let deserializer = Deserializer::new(content).coerce_numbers();
    T::deserialize(deserializer).map_err(Into::into)
} */
