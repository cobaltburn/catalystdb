use crate::{
    dbs::graph::Graph,
    doc::document::Cursor,
    err::Error,
    ql::{operator::Operator, traits::Incoperate, value::Value},
};
use actix::Addr;
use reblessive::tree::Stk;
use std::{collections::BTreeMap, fmt, ops::Deref, sync::Arc};

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
    pub async fn evaluate(
        &self,
        stk: &mut Stk,
        graph: &Addr<Graph>,
        cur: Option<&Cursor>,
    ) -> Result<Value, Error> {
        Ok(match self {
            Expression::Unary { op, expr } => match op {
                Operator::Neg => stk
                    .run(|stk| expr.evaluate(stk, graph, cur))
                    .await?
                    .try_neg()?,
                Operator::Not => stk
                    .run(|stk| expr.evaluate(stk, graph, cur))
                    .await?
                    .try_not()?,
                op => return Err(Error::InvalidOperator(op.clone())),
            },
            Expression::Binary { left, op, right } => {
                let left = stk.run(|stk| left.evaluate(stk, graph, cur)).await?;
                let right = stk.run(|stk| right.evaluate(stk, graph, cur)).await?;
                match op {
                    Operator::Or => Value::Bool(left.is_truthy() || right.is_truthy()),
                    Operator::And => Value::Bool(left.is_truthy() && right.is_truthy()),
                    Operator::Eq => Value::Bool(left.eq(&right)),
                    Operator::NtEq => Value::Bool(left.ne(&right)),
                    Operator::Lt => Value::Bool(left.lt(&right)),
                    Operator::Gt => Value::Bool(left.gt(&right)),
                    Operator::LtEq => Value::Bool(left.le(&right)),
                    Operator::GtEq => Value::Bool(left.ge(&right)),
                    Operator::Add => left.try_add(right)?,
                    Operator::Sub => left.try_sub(right)?,
                    Operator::Mult => left.try_mul(right)?,
                    Operator::Div => left.try_div(right)?,
                    op => return Err(Error::InvalidOperator(op.clone())),
                }
            }
        })
    }
}

impl Incoperate for Expression {
    fn incorperate(&self, fields: &BTreeMap<Arc<str>, Value>) -> Expression {
        match self.to_owned() {
            Expression::Unary {
                op,
                expr: Value::Idiom(v),
            } => Expression::Unary {
                op,
                expr: v.incorperate(fields).into(),
            },
            Expression::Unary {
                op,
                expr: Value::Expression(v),
            } => Expression::Unary {
                op,
                expr: v.deref().incorperate(fields).into(),
            },
            Expression::Binary {
                left: Value::Idiom(l),
                op,
                right: Value::Idiom(r),
            } => Expression::Binary {
                left: l.incorperate(fields).into(),
                op,
                right: r.incorperate(fields).into(),
            },
            Expression::Binary {
                left: Value::Idiom(l),
                op,
                right,
            } => Expression::Binary {
                left: l.incorperate(fields).into(),
                op,
                right,
            },
            Expression::Binary {
                left,
                op,
                right: Value::Idiom(r),
            } => Expression::Binary {
                left,
                op,
                right: r.incorperate(fields).into(),
            },
            Expression::Binary {
                left: Value::Expression(l),
                op,
                right: Value::Expression(r),
            } => Expression::Binary {
                left: l.incorperate(fields).into(),
                op,
                right: r.incorperate(fields).into(),
            },
            Expression::Binary {
                left: Value::Expression(l),
                op,
                right,
            } => Expression::Binary {
                left: l.incorperate(fields).into(),
                op,
                right,
            },
            Expression::Binary {
                left,
                op,
                right: Value::Expression(r),
            } => Expression::Binary {
                left,
                op,
                right: r.incorperate(fields).into(),
            },
            v => v,
        }
    }
}

// #[cfg(test)]
/* mod test {
    use std::collections::BTreeMap;

    use super::*;
    use crate::ql::{array::Array, number::Number, object::Object, record::Record};

    #[test]
    fn test_not_expression_false() {
        let false_exp = Expression::Unary {
            op: Operator::Not,
            expr: Value::Bool(true),
        };
        let false_eval = false_exp.evaluate(&Value::None).unwrap();

        assert_eq!(false_eval, Value::Bool(false));
    }

    #[test]
    fn test_not_expression_true() {
        let true_exp = Expression::Unary {
            op: Operator::Not,
            expr: Value::Bool(false),
        };
        let true_eval = true_exp.evaluate(&Value::None).unwrap();

        assert_eq!(true_eval, Value::Bool(true));
    }

    #[test]
    fn test_or_expression() {
        let expressions = Vec::from([
            Expression::Binary {
                left: true.into(),
                op: Operator::Or,
                right: true.into(),
            },
            Expression::Binary {
                left: false.into(),
                op: Operator::Or,
                right: true.into(),
            },
            Expression::Binary {
                left: true.into(),
                op: Operator::Or,
                right: false.into(),
            },
            Expression::Binary {
                left: false.into(),
                op: Operator::Or,
                right: false.into(),
            },
        ]);

        let expressions: Vec<Value> = expressions
            .into_iter()
            .map(|expr| expr.evaluate(&Value::None).unwrap())
            .collect();

        assert_eq!(expressions[0], Value::Bool(true));
        assert_eq!(expressions[1], Value::Bool(true));
        assert_eq!(expressions[2], Value::Bool(true));
        assert_eq!(expressions[3], Value::Bool(false));
    }

    #[test]
    fn test_and_expression() {
        let expressions = Vec::from([
            Expression::Binary {
                left: true.into(),
                op: Operator::And,
                right: true.into(),
            },
            Expression::Binary {
                left: false.into(),
                op: Operator::And,
                right: true.into(),
            },
            Expression::Binary {
                left: true.into(),
                op: Operator::And,
                right: false.into(),
            },
            Expression::Binary {
                left: false.into(),
                op: Operator::And,
                right: false.into(),
            },
        ]);

        let expressions: Vec<Value> = expressions
            .into_iter()
            .map(|expr| expr.evaluate(&Value::None).unwrap())
            .collect();

        assert_eq!(expressions[0], Value::Bool(true));
        assert_eq!(expressions[1], Value::Bool(false));
        assert_eq!(expressions[2], Value::Bool(false));
        assert_eq!(expressions[3], Value::Bool(false));
    }

    #[test]
    fn test_bool_eq_expression() {
        let expressions = Vec::from([
            Expression::Binary {
                left: true.into(),
                op: Operator::Eq,
                right: true.into(),
            },
            Expression::Binary {
                left: false.into(),
                op: Operator::Eq,
                right: true.into(),
            },
            Expression::Binary {
                left: true.into(),
                op: Operator::Eq,
                right: false.into(),
            },
            Expression::Binary {
                left: false.into(),
                op: Operator::Eq,
                right: false.into(),
            },
        ]);

        let expressions: Vec<Value> = expressions
            .into_iter()
            .map(|expr| expr.evaluate(&Value::None).unwrap())
            .collect();

        assert_eq!(expressions[0], Value::Bool(true));
        assert_eq!(expressions[1], Value::Bool(false));
        assert_eq!(expressions[2], Value::Bool(false));
        assert_eq!(expressions[3], Value::Bool(true));
    }

    #[test]
    fn test_num_eq_expression() {
        let expressions = Vec::from([
            Expression::Binary {
                left: 1.into(),
                op: Operator::Eq,
                right: 1.into(),
            },
            Expression::Binary {
                left: 1.into(),
                op: Operator::Eq,
                right: 2.into(),
            },
            Expression::Binary {
                left: 2.into(),
                op: Operator::Eq,
                right: 1.into(),
            },
        ]);

        let expressions: Vec<Value> = expressions
            .into_iter()
            .map(|expr| expr.evaluate(&Value::None).unwrap())
            .collect();

        assert_eq!(expressions[0], Value::Bool(true));
        assert_eq!(expressions[1], Value::Bool(false));
        assert_eq!(expressions[2], Value::Bool(false));
    }

    #[test]
    fn test_string_eq_expression() {
        let expressions = Vec::from([
            Expression::Binary {
                left: "a".into(),
                op: Operator::Eq,
                right: "a".into(),
            },
            Expression::Binary {
                left: "b".into(),
                op: Operator::Eq,
                right: "a".into(),
            },
            Expression::Binary {
                left: "a".into(),
                op: Operator::Eq,
                right: "b".into(),
            },
        ]);

        let expressions: Vec<Value> = expressions
            .into_iter()
            .map(|expr| expr.evaluate(&Value::None).unwrap())
            .collect();

        assert_eq!(expressions[0], Value::Bool(true));
        assert_eq!(expressions[1], Value::Bool(false));
        assert_eq!(expressions[2], Value::Bool(false));
    }

    #[test]
    fn test_object_eq_expression() {
        let obj1 = Object(BTreeMap::from([
            ("b".into(), 2.into()),
            ("id".into(), Record::new("table", 1).into()),
        ]));
        let obj2 = Object(BTreeMap::from([
            ("a".into(), 2.into()),
            ("id".into(), Record::new("table", 1).into()),
        ]));
        let expressions = Vec::from([
            Expression::Binary {
                left: Value::Object(obj1.clone()),
                op: Operator::Eq,
                right: Value::Object(obj1.clone()),
            },
            Expression::Binary {
                left: Value::Object(obj1.clone()),
                op: Operator::Eq,
                right: Value::Object(obj2.clone()),
            },
            Expression::Binary {
                left: Value::Object(obj2.clone()),
                op: Operator::Eq,
                right: Value::Object(obj1.clone()),
            },
        ]);

        let expressions: Vec<Value> = expressions
            .into_iter()
            .map(|expr| expr.evaluate(&Value::None).unwrap())
            .collect();

        assert_eq!(expressions[0], Value::Bool(true));
        assert_eq!(expressions[1], Value::Bool(false));
        assert_eq!(expressions[2], Value::Bool(false));
    }

    #[test]
    fn test_record_eq_expression() {
        let r1 = Record::new("a", 1);
        let r2 = Record::new("b", 2);
        let expressions = Vec::from([
            Expression::Binary {
                left: Value::Record(Box::new(r1.clone())),
                op: Operator::Eq,
                right: Value::Record(Box::new(r1.clone())),
            },
            Expression::Binary {
                left: Value::Record(Box::new(r1.clone())),
                op: Operator::Eq,
                right: Value::Record(Box::new(r2.clone())),
            },
            Expression::Binary {
                left: Value::Record(Box::new(r2.clone())),
                op: Operator::Eq,
                right: Value::Record(Box::new(r1.clone())),
            },
        ]);

        let expressions: Vec<Value> = expressions
            .into_iter()
            .map(|expr| expr.evaluate(&Value::None).unwrap())
            .collect();

        assert_eq!(expressions[0], Value::Bool(true));
        assert_eq!(expressions[1], Value::Bool(false));
        assert_eq!(expressions[2], Value::Bool(false));
    }

    #[test]
    fn test_array_eq_expression() {
        let array1 = Array(vec![Value::None]);
        let array2 = Array(vec![Value::None, Value::None, Value::Null]);
        let expressions = Vec::from([
            Expression::Binary {
                left: Value::Array(array1.clone()),
                op: Operator::Eq,
                right: Value::Array(array1.clone()),
            },
            Expression::Binary {
                left: Value::Array(array1.clone()),
                op: Operator::Eq,
                right: Value::Array(array2.clone()),
            },
            Expression::Binary {
                left: Value::Array(array2.clone()),
                op: Operator::Eq,
                right: Value::Array(array1.clone()),
            },
        ]);

        let expressions: Vec<Value> = expressions
            .into_iter()
            .map(|expr| expr.evaluate(&Value::None).unwrap())
            .collect();

        assert_eq!(expressions[0], Value::Bool(true));
        assert_eq!(expressions[1], Value::Bool(false));
        assert_eq!(expressions[2], Value::Bool(false));

        let array1 = Array(vec![Value::Number(1.into())]);
        let array2 = Array(vec![Value::Number(2.into())]);
        let expressions = Vec::from([
            Expression::Binary {
                left: Value::Array(array1.clone()),
                op: Operator::Eq,
                right: Value::Array(array1.clone()),
            },
            Expression::Binary {
                left: Value::Array(array1.clone()),
                op: Operator::Eq,
                right: Value::Array(array2.clone()),
            },
            Expression::Binary {
                left: Value::Array(array2.clone()),
                op: Operator::Eq,
                right: Value::Array(array1.clone()),
            },
        ]);

        let expressions: Vec<Value> = expressions
            .into_iter()
            .map(|expr| expr.evaluate(&Value::None).unwrap())
            .collect();

        assert_eq!(expressions[0], Value::Bool(true));
        assert_eq!(expressions[1], Value::Bool(false));
        assert_eq!(expressions[2], Value::Bool(false));
    }

    #[test]
    fn test_bool_nteq_expression() {
        let expressions = Vec::from([
            Expression::Binary {
                left: true.into(),
                op: Operator::NtEq,
                right: true.into(),
            },
            Expression::Binary {
                left: false.into(),
                op: Operator::NtEq,
                right: true.into(),
            },
            Expression::Binary {
                left: true.into(),
                op: Operator::NtEq,
                right: false.into(),
            },
            Expression::Binary {
                left: false.into(),
                op: Operator::NtEq,
                right: false.into(),
            },
        ]);

        let expressions: Vec<Value> = expressions
            .into_iter()
            .map(|expr| expr.evaluate(&Value::None).unwrap())
            .collect();

        assert_eq!(expressions[0], Value::Bool(false));
        assert_eq!(expressions[1], Value::Bool(true));
        assert_eq!(expressions[2], Value::Bool(true));
        assert_eq!(expressions[3], Value::Bool(false));
    }

    #[test]
    fn test_num_nteq_expression() {
        let expressions = Vec::from([
            Expression::Binary {
                left: 1.into(),
                op: Operator::NtEq,
                right: 1.into(),
            },
            Expression::Binary {
                left: 1.into(),
                op: Operator::NtEq,
                right: 2.into(),
            },
            Expression::Binary {
                left: 2.into(),
                op: Operator::NtEq,
                right: 1.into(),
            },
        ]);

        let expressions: Vec<Value> = expressions
            .into_iter()
            .map(|expr| expr.evaluate(&Value::None).unwrap())
            .collect();

        assert_eq!(expressions[0], Value::Bool(false));
        assert_eq!(expressions[1], Value::Bool(true));
        assert_eq!(expressions[2], Value::Bool(true));
    }

    #[test]
    fn test_string_nteq_expression() {
        let expressions = Vec::from([
            Expression::Binary {
                left: "a".into(),
                op: Operator::NtEq,
                right: "a".into(),
            },
            Expression::Binary {
                left: "b".into(),
                op: Operator::NtEq,
                right: "a".into(),
            },
            Expression::Binary {
                left: "a".into(),
                op: Operator::NtEq,
                right: "b".into(),
            },
        ]);

        let expressions: Vec<Value> = expressions
            .into_iter()
            .map(|expr| expr.evaluate(&Value::None).unwrap())
            .collect();

        assert_eq!(expressions[0], Value::Bool(false));
        assert_eq!(expressions[1], Value::Bool(true));
        assert_eq!(expressions[2], Value::Bool(true));
    }

    #[test]
    fn test_object_nteq_expression() {
        let obj1 = Object(BTreeMap::from([
            ("b".into(), 2.into()),
            ("id".into(), Record::new("table", 1).into()),
        ]));
        let obj2 = Object(BTreeMap::from([
            ("a".into(), 2.into()),
            ("id".into(), Record::new("table", 1).into()),
        ]));
        let expressions = Vec::from([
            Expression::Binary {
                left: Value::Object(obj1.clone()),
                op: Operator::NtEq,
                right: Value::Object(obj1.clone()),
            },
            Expression::Binary {
                left: Value::Object(obj1.clone()),
                op: Operator::NtEq,
                right: Value::Object(obj2.clone()),
            },
            Expression::Binary {
                left: Value::Object(obj2.clone()),
                op: Operator::NtEq,
                right: Value::Object(obj1.clone()),
            },
        ]);

        let expressions: Vec<Value> = expressions
            .into_iter()
            .map(|expr| expr.evaluate(&Value::Object(Object::default())).unwrap())
            .collect();

        assert_eq!(expressions[0], Value::Bool(false));
        assert_eq!(expressions[1], Value::Bool(true));
        assert_eq!(expressions[2], Value::Bool(true));
    }

    #[test]
    fn test_record_nteq_expression() {
        let r1 = Record::new("a", 1);
        let r2 = Record::new("b", 2);
        let expressions = Vec::from([
            Expression::Binary {
                left: Value::Record(Box::new(r1.clone())),
                op: Operator::NtEq,
                right: Value::Record(Box::new(r1.clone())),
            },
            Expression::Binary {
                left: Value::Record(Box::new(r1.clone())),
                op: Operator::NtEq,
                right: Value::Record(Box::new(r2.clone())),
            },
            Expression::Binary {
                left: Value::Record(Box::new(r2.clone())),
                op: Operator::NtEq,
                right: Value::Record(Box::new(r1.clone())),
            },
        ]);

        let expressions: Vec<Value> = expressions
            .into_iter()
            .map(|expr| expr.evaluate(&Value::Object(Object::default())).unwrap())
            .collect();

        assert_eq!(expressions[0], Value::Bool(false));
        assert_eq!(expressions[1], Value::Bool(true));
        assert_eq!(expressions[2], Value::Bool(true));
    }

    #[test]
    fn test_array_nteq_expression() {
        let array1 = Array(vec![Value::None]);
        let array2 = Array(vec![Value::None, Value::None, Value::Null]);
        let expressions = Vec::from([
            Expression::Binary {
                left: Value::Array(array1.clone()),
                op: Operator::NtEq,
                right: Value::Array(array1.clone()),
            },
            Expression::Binary {
                left: Value::Array(array1.clone()),
                op: Operator::NtEq,
                right: Value::Array(array2.clone()),
            },
            Expression::Binary {
                left: Value::Array(array2.clone()),
                op: Operator::NtEq,
                right: Value::Array(array1.clone()),
            },
        ]);

        let expressions: Vec<Value> = expressions
            .into_iter()
            .map(|expr| expr.evaluate(&Value::Object(Object::default())).unwrap())
            .collect();

        assert_eq!(expressions[0], Value::Bool(false));
        assert_eq!(expressions[1], Value::Bool(true));
        assert_eq!(expressions[2], Value::Bool(true));

        let array1 = Array(vec![Value::Number(1.into())]);
        let array2 = Array(vec![Value::Number(2.into())]);
        let expressions = Vec::from([
            Expression::Binary {
                left: Value::Array(array1.clone()),
                op: Operator::NtEq,
                right: Value::Array(array1.clone()),
            },
            Expression::Binary {
                left: Value::Array(array1.clone()),
                op: Operator::NtEq,
                right: Value::Array(array2.clone()),
            },
            Expression::Binary {
                left: Value::Array(array2.clone()),
                op: Operator::NtEq,
                right: Value::Array(array1.clone()),
            },
        ]);

        let expressions: Vec<Value> = expressions
            .into_iter()
            .map(|expr| expr.evaluate(&Value::Object(Object::default())).unwrap())
            .collect();

        assert_eq!(expressions[0], Value::Bool(false));
        assert_eq!(expressions[1], Value::Bool(true));
        assert_eq!(expressions[2], Value::Bool(true));
    }

    #[test]
    fn test_int_lt_expression() {
        let expressions = Vec::from([
            Expression::Binary {
                left: 1.into(),
                op: Operator::Lt,
                right: 1.into(),
            },
            Expression::Binary {
                left: 1.into(),
                op: Operator::Lt,
                right: 2.into(),
            },
            Expression::Binary {
                left: 2.into(),
                op: Operator::Lt,
                right: 1.into(),
            },
        ]);

        let expressions: Vec<Value> = expressions
            .into_iter()
            .map(|expr| expr.evaluate(&Value::Object(Object::default())).unwrap())
            .collect();

        assert_eq!(expressions[0], Value::Bool(false));
        assert_eq!(expressions[1], Value::Bool(true));
        assert_eq!(expressions[2], Value::Bool(false));
    }

    #[test]
    fn test_float_lt_expression() {
        let expressions = Vec::from([
            Expression::Binary {
                left: Value::Number(Number::Float(1.0)),
                op: Operator::Lt,
                right: Value::Number(Number::Float(1.0)),
            },
            Expression::Binary {
                left: Value::Number(Number::Float(1.5)),
                op: Operator::Lt,
                right: Value::Number(Number::Float(2.0)),
            },
            Expression::Binary {
                left: Value::Number(Number::Float(2.0)),
                op: Operator::Lt,
                right: Value::Number(Number::Float(1.0)),
            },
        ]);

        let expressions: Vec<Value> = expressions
            .into_iter()
            .map(|expr| expr.evaluate(&Value::Object(Object::default())).unwrap())
            .collect();

        assert_eq!(expressions[0], Value::Bool(false));
        assert_eq!(expressions[1], Value::Bool(true));
        assert_eq!(expressions[2], Value::Bool(false));
    }

    #[test]
    fn test_number_lt_expression() {
        let expressions = Vec::from([
            Expression::Binary {
                left: Value::Number(Number::Float(1.0)),
                op: Operator::Lt,
                right: Value::Number(Number::Int(1)),
            },
            Expression::Binary {
                left: Value::Number(Number::Float(1.5)),
                op: Operator::Lt,
                right: Value::Number(Number::Int(2)),
            },
            Expression::Binary {
                left: Value::Number(Number::Float(2.0)),
                op: Operator::Lt,
                right: Value::Number(Number::Int(1)),
            },
        ]);

        let expressions: Vec<Value> = expressions
            .into_iter()
            .map(|expr| expr.evaluate(&Value::Object(Object::default())).unwrap())
            .collect();

        assert_eq!(expressions[0], Value::Bool(false));
        assert_eq!(expressions[1], Value::Bool(true));
        assert_eq!(expressions[2], Value::Bool(false));
    }

    #[test]
    fn test_int_lteq_expression() {
        let expressions = Vec::from([
            Expression::Binary {
                left: 1.into(),
                op: Operator::LtEq,
                right: 1.into(),
            },
            Expression::Binary {
                left: 1.into(),
                op: Operator::LtEq,
                right: 2.into(),
            },
            Expression::Binary {
                left: 2.into(),
                op: Operator::LtEq,
                right: 1.into(),
            },
        ]);

        let expressions: Vec<Value> = expressions
            .into_iter()
            .map(|expr| expr.evaluate(&Value::Object(Object::default())).unwrap())
            .collect();

        assert_eq!(expressions[0], Value::Bool(true));
        assert_eq!(expressions[1], Value::Bool(true));
        assert_eq!(expressions[2], Value::Bool(false));
    }

    #[test]
    fn test_float_lteq_expression() {
        let expressions = Vec::from([
            Expression::Binary {
                left: Value::Number(Number::Float(1.0)),
                op: Operator::LtEq,
                right: Value::Number(Number::Float(1.0)),
            },
            Expression::Binary {
                left: Value::Number(Number::Float(1.5)),
                op: Operator::LtEq,
                right: Value::Number(Number::Float(2.0)),
            },
            Expression::Binary {
                left: Value::Number(Number::Float(2.0)),
                op: Operator::LtEq,
                right: Value::Number(Number::Float(1.0)),
            },
        ]);

        let expressions: Vec<Value> = expressions
            .into_iter()
            .map(|expr| expr.evaluate(&Value::Object(Object::default())).unwrap())
            .collect();

        assert_eq!(expressions[0], Value::Bool(true));
        assert_eq!(expressions[1], Value::Bool(true));
        assert_eq!(expressions[2], Value::Bool(false));
    }

    #[test]
    fn test_number_lteq_expression() {
        let expressions = Vec::from([
            Expression::Binary {
                left: Value::Number(Number::Float(1.0)),
                op: Operator::LtEq,
                right: Value::Number(Number::Int(1)),
            },
            Expression::Binary {
                left: Value::Number(Number::Float(1.5)),
                op: Operator::LtEq,
                right: Value::Number(Number::Int(2)),
            },
            Expression::Binary {
                left: Value::Number(Number::Float(2.0)),
                op: Operator::LtEq,
                right: Value::Number(Number::Int(1)),
            },
        ]);

        let expressions: Vec<Value> = expressions
            .into_iter()
            .map(|expr| expr.evaluate(&Value::Object(Object::default())).unwrap())
            .collect();

        assert_eq!(expressions[0], Value::Bool(true));
        assert_eq!(expressions[1], Value::Bool(true));
        assert_eq!(expressions[2], Value::Bool(false));
    }

    // TODO need to change
    #[test]
    fn test_int_gt_expression() {
        let expressions = Vec::from([
            Expression::Binary {
                left: 1.into(),
                op: Operator::Gt,
                right: 1.into(),
            },
            Expression::Binary {
                left: 1.into(),
                op: Operator::Gt,
                right: 2.into(),
            },
            Expression::Binary {
                left: 2.into(),
                op: Operator::Gt,
                right: 1.into(),
            },
        ]);

        let expressions: Vec<Value> = expressions
            .into_iter()
            .map(|expr| expr.evaluate(&Value::Object(Object::default())).unwrap())
            .collect();

        assert_eq!(expressions[0], Value::Bool(false));
        assert_eq!(expressions[1], Value::Bool(false));
        assert_eq!(expressions[2], Value::Bool(true));
    }

    #[test]
    fn test_float_gt_expression() {
        let expressions = Vec::from([
            Expression::Binary {
                left: Value::Number(Number::Float(1.0)),
                op: Operator::Gt,
                right: Value::Number(Number::Float(1.0)),
            },
            Expression::Binary {
                left: Value::Number(Number::Float(1.5)),
                op: Operator::Gt,
                right: Value::Number(Number::Float(2.0)),
            },
            Expression::Binary {
                left: Value::Number(Number::Float(2.0)),
                op: Operator::Gt,
                right: Value::Number(Number::Float(1.0)),
            },
        ]);

        let expressions: Vec<Value> = expressions
            .into_iter()
            .map(|expr| expr.evaluate(&Value::Object(Object::default())).unwrap())
            .collect();

        assert_eq!(expressions[0], Value::Bool(false));
        assert_eq!(expressions[1], Value::Bool(false));
        assert_eq!(expressions[2], Value::Bool(true));
    }

    #[test]
    fn test_number_gt_expression() {
        let expressions = Vec::from([
            Expression::Binary {
                left: Value::Number(Number::Float(1.0)),
                op: Operator::Gt,
                right: Value::Number(Number::Int(1)),
            },
            Expression::Binary {
                left: Value::Number(Number::Float(1.5)),
                op: Operator::Gt,
                right: Value::Number(Number::Int(2)),
            },
            Expression::Binary {
                left: Value::Number(Number::Float(2.0)),
                op: Operator::Gt,
                right: Value::Number(Number::Int(1)),
            },
        ]);

        let expressions: Vec<Value> = expressions
            .into_iter()
            .map(|expr| expr.evaluate(&Value::Object(Object::default())).unwrap())
            .collect();

        assert_eq!(expressions[0], Value::Bool(false));
        assert_eq!(expressions[1], Value::Bool(false));
        assert_eq!(expressions[2], Value::Bool(true));
    }

    #[test]
    fn test_int_gteq_expression() {
        let expressions = Vec::from([
            Expression::Binary {
                left: 1.into(),
                op: Operator::GtEq,
                right: 1.into(),
            },
            Expression::Binary {
                left: 1.into(),
                op: Operator::GtEq,
                right: 2.into(),
            },
            Expression::Binary {
                left: 2.into(),
                op: Operator::GtEq,
                right: 1.into(),
            },
        ]);

        let expressions: Vec<Value> = expressions
            .into_iter()
            .map(|expr| expr.evaluate(&Value::Object(Object::default())).unwrap())
            .collect();

        assert_eq!(expressions[0], Value::Bool(true));
        assert_eq!(expressions[1], Value::Bool(false));
        assert_eq!(expressions[2], Value::Bool(true));
    }

    #[test]
    fn test_float_gteq_expression() {
        let expressions = Vec::from([
            Expression::Binary {
                left: Value::Number(Number::Float(1.0)),
                op: Operator::GtEq,
                right: Value::Number(Number::Float(1.0)),
            },
            Expression::Binary {
                left: Value::Number(Number::Float(1.5)),
                op: Operator::GtEq,
                right: Value::Number(Number::Float(2.0)),
            },
            Expression::Binary {
                left: Value::Number(Number::Float(2.0)),
                op: Operator::GtEq,
                right: Value::Number(Number::Float(1.0)),
            },
        ]);

        let expressions: Vec<Value> = expressions
            .into_iter()
            .map(|expr| expr.evaluate(&Value::Object(Object::default())).unwrap())
            .collect();

        assert_eq!(expressions[0], Value::Bool(true));
        assert_eq!(expressions[1], Value::Bool(false));
        assert_eq!(expressions[2], Value::Bool(true));
    }

    #[test]
    fn test_number_gteq_expression() {
        let expressions = Vec::from([
            Expression::Binary {
                left: Value::Number(Number::Float(1.0)),
                op: Operator::GtEq,
                right: Value::Number(Number::Int(1)),
            },
            Expression::Binary {
                left: Value::Number(Number::Float(1.5)),
                op: Operator::GtEq,
                right: Value::Number(Number::Int(2)),
            },
            Expression::Binary {
                left: Value::Number(Number::Float(2.0)),
                op: Operator::GtEq,
                right: Value::Number(Number::Int(1)),
            },
        ]);

        let expressions: Vec<Value> = expressions
            .into_iter()
            .map(|expr| expr.evaluate(&Value::Object(Object::default())).unwrap())
            .collect();

        assert_eq!(expressions[0], Value::Bool(true));
        assert_eq!(expressions[1], Value::Bool(false));
        assert_eq!(expressions[2], Value::Bool(true));
    }
} */
