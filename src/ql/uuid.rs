use std::fmt::{self, Display};

#[derive(Debug, Clone, PartialEq, Hash, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub struct Uuid(pub uuid::Uuid);

impl fmt::Display for Uuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "u")?;
        Display::fmt(&self.0.to_string(), f)
    }
}
