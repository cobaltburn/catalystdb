use core::fmt;
use std::ops::Deref;

#[derive(Debug, Clone, Default, PartialEq, Hash, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub struct Tables(pub Vec<Table>);

impl fmt::Display for Tables {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, v) in self.0.iter().enumerate() {
            if i > 0 {
                f.write_str(", ")?;
            }
            write!(f, "{}", &v)?;
        }
        Ok(())
    }
}

impl Deref for Tables {
    type Target = Vec<Table>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Default, PartialEq, Hash, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub struct Table(pub String);

impl From<String> for Table {
    fn from(table: String) -> Self {
        Table(table)
    }
}

impl fmt::Display for Table {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Deref for Table {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
