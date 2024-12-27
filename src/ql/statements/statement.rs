use crate::{
    dbs::iterator::Iterable,
    err::Error,
    ql::{
        condition::Condition,
        fields::Fields,
        statements::select::Select,
        value::{Value, Values},
    },
};

pub enum Statement<'a> {
    Select(&'a Select),
}

impl<'a> From<&'a Select> for Statement<'a> {
    fn from(select: &'a Select) -> Self {
        Statement::Select(select)
    }
}

impl Statement<'_> {
    pub fn is_select(&self) -> bool {
        matches!(self, Statement::Select(_))
    }

    pub fn fields(&self) -> Option<&Fields> {
        Some(match self {
            Statement::Select(stm) => &stm.fields,
        })
    }

    pub fn what(&self) -> Option<&Values> {
        Some(match self {
            Statement::Select(stm) => &stm.what,
        })
    }

    pub fn filter(&self) -> Option<&Condition> {
        match self {
            Statement::Select(stm) => stm.filter.as_ref(),
        }
    }

    pub fn limit(&self) -> Option<&u32> {
        match self {
            Statement::Select(stm) => stm.limit.as_ref(),
        }
    }

    pub fn start(&self) -> Option<&u32> {
        match self {
            Statement::Select(stm) => stm.start.as_ref(),
        }
    }
}
