use std::sync::Arc;

#[derive(Debug, Clone, Default, PartialEq, Hash, Eq, PartialOrd, Ord)]
pub struct Table(pub Arc<str>);
