use crate::{
    dbs::node::Node,
    err::Error,
    func::operate,
    ql::{operator::Operator, value::Value},
};
use std::{collections::BTreeMap, fmt, sync::Arc};

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum Expression {
    Unary {
        op: Operator,
        expr: Value,
    },
    Binary {
        left: Value,
        op: Operator,
        right: Value,
    },
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expression::Unary { op, expr } => write!(f, "{op}{expr}"),
            Expression::Binary { left, op, right } => write!(f, "{left} {op} {right}"),
        }
    }
}

impl Expression {
    pub fn evaluate(&self, node: &Node) -> Result<Value, Error> {
        match self {
            Expression::Unary { op, expr } => match op {
                Operator::Neg => operate::neg(expr.evaluate(node)?),
                Operator::Not => operate::not(expr.evaluate(node)?),
                op => Err(Error::InvalidOperator(op.clone())),
            },
            Expression::Binary { left, op, right } => {
                let left = left.evaluate(node)?;
                let right = right.evaluate(node)?;
                match op {
                    Operator::Or => todo!(),
                    Operator::And => todo!(),
                    Operator::Add => todo!(),
                    Operator::Sub => todo!(),
                    Operator::Mult => todo!(),
                    Operator::Div => todo!(),
                    Operator::Eq => todo!(),
                    Operator::Lt => todo!(),
                    Operator::Gt => todo!(),
                    Operator::LtEq => todo!(),
                    Operator::GtEq => todo!(),
                    op => Err(Error::InvalidOperator(op.clone())),
                }
            }
        }
    }
}
