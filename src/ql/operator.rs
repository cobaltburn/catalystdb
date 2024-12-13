use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
#[non_exhaustive]
pub enum Operator {
    Neg,
    Not,

    Or,
    And,

    Add,
    Sub,
    Mult,
    Div,

    Eq,
    NtEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operator::Neg => write!(f, "-"),
            Operator::Not => write!(f, "!"),
            Operator::Or => write!(f, "OR"),
            Operator::And => write!(f, "AND"),
            Operator::Add => write!(f, "+"),
            Operator::Sub => write!(f, "-"),
            Operator::Mult => write!(f, "*"),
            Operator::Div => write!(f, "/"),
            Operator::Eq => write!(f, "="),
            Operator::Lt => write!(f, "<"),
            Operator::Gt => write!(f, ">"),
            Operator::LtEq => write!(f, "<="),
            Operator::GtEq => write!(f, ">="),
            Operator::NtEq => write!(f, "!="),
        }
    }
}
