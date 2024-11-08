use crate::{err::Error, ql::value::Value};

pub fn neg(val: Value) -> Result<Value, Error> {
    val.try_neg()
}

pub fn not(val: Value) -> Result<Value, Error> {
    Ok(val.is_truthy().into())
}
