use std::{fmt, sync::Arc};

#[derive(Debug, Clone, Default, PartialEq, Hash, Eq, PartialOrd, Ord)]
pub struct Ident(pub Arc<str>);

impl Ident {
    pub fn new<S: Into<Arc<str>>>(ident: S) -> Self {
        Ident(ident.into())
    }

    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl fmt::Display for Ident {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
