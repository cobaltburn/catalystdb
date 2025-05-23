use std::{
    fmt::Display,
    ops::{self, Deref},
    sync::Arc,
};

#[derive(Debug, Clone, Default, PartialEq, Hash, Eq, PartialOrd, Ord)]
pub struct Strand(pub Arc<str>);

impl Deref for Strand {
    type Target = Arc<str>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<String> for Strand {
    fn from(s: String) -> Self {
        Strand(s.into())
    }
}

impl From<&str> for Strand {
    fn from(s: &str) -> Self {
        Strand(s.into())
    }
}

impl Strand {
    pub fn new<T: Into<Arc<str>>>(val: T) -> Strand {
        Strand(val.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn as_string(&self) -> String {
        self.0.deref().into()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl ops::Add for Strand {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let l = self.0.deref();
        let r = rhs.0.deref();
        Strand(format!("{l}{r}").into())
    }
}

impl Display for Strand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
