use core::fmt;
use std::ops::Deref;

use crate::ql::value::Value;

#[derive(Debug, Clone, Default, PartialEq, Hash, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub struct Condition(pub Value);

impl Deref for Condition {
    type Target = Value;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for Condition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WHERE {}", self.0)
    }
}
