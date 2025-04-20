use crate::err::Error;
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    fmt::Display,
    hash,
    ops::{self},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Number {
    Int(i64),
    Float(f64),
}

impl Default for Number {
    fn default() -> Self {
        Number::Int(0)
    }
}

impl From<f32> for Number {
    fn from(value: f32) -> Self {
        Number::Float(value as f64)
    }
}

impl From<f64> for Number {
    fn from(value: f64) -> Self {
        Number::Float(value)
    }
}

impl From<i8> for Number {
    fn from(value: i8) -> Self {
        Number::Int(value as i64)
    }
}

impl From<i16> for Number {
    fn from(value: i16) -> Self {
        Number::Int(value as i64)
    }
}

impl From<i32> for Number {
    fn from(value: i32) -> Self {
        Number::Int(value as i64)
    }
}

impl From<i64> for Number {
    fn from(value: i64) -> Self {
        Number::Int(value)
    }
}

impl From<u8> for Number {
    fn from(value: u8) -> Self {
        Number::Int(value as i64)
    }
}

impl From<u16> for Number {
    fn from(value: u16) -> Self {
        Number::Int(value as i64)
    }
}

impl From<u32> for Number {
    fn from(value: u32) -> Self {
        Number::Int(value as i64)
    }
}

impl From<u64> for Number {
    fn from(value: u64) -> Self {
        Number::Int(value as i64)
    }
}

impl Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Number::Int(v) => write!(f, "{v}"),
            Number::Float(v) => write!(f, "{v}"),
        }
    }
}

impl hash::Hash for Number {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        match self {
            Number::Int(num) => num.hash(state),
            Number::Float(num) => num.to_bits().hash(state),
        }
    }
}

impl PartialEq for Number {
    fn eq(&self, other: &Self) -> bool {
        fn total_eq_f64(a: f64, b: f64) -> bool {
            a.to_bits().eq(&b.to_bits()) || (a == 0.0 && b == 0.0)
        }

        match (self, other) {
            (Number::Int(l), Number::Int(r)) => l.eq(r),
            (Number::Float(l), Number::Float(r)) => total_eq_f64(*l, *r),
            (l @ Number::Int(_), r @ Number::Float(_)) => l.cmp(r) == Ordering::Equal,
            (l @ Number::Float(_), r @ Number::Int(_)) => l.cmp(r) == Ordering::Equal,
        }
    }
}

impl PartialOrd for Number {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Number {}

impl Ord for Number {
    fn cmp(&self, other: &Self) -> Ordering {
        fn cmp_f64(a: f64, b: f64) -> Ordering {
            if a == 0.0 && b == 0.0 {
                Ordering::Equal
            } else {
                a.total_cmp(&b)
            }
        }

        macro_rules! greater {
            ($f:ident) => {
                if $f.is_sign_positive() {
                    Ordering::Greater
                } else {
                    Ordering::Less
                }
            };
        }

        match (self, other) {
            (Number::Int(a), Number::Int(b)) => a.cmp(b),
            (Number::Float(a), Number::Float(b)) => cmp_f64(*a, *b),
            (Number::Int(a), Number::Float(b)) => {
                if !b.is_finite() {
                    return greater!(b).reverse();
                }
                let l = *a as i128;
                let r = *b as i128;
                match l.cmp(&r) {
                    Ordering::Equal => cmp_f64(0.0, b.fract()),
                    ordering => ordering,
                }
            }
            (a @ Number::Float(_), b @ Number::Int(_)) => b.cmp(a).reverse(),
        }
    }
}

impl ops::Add for Number {
    type Output = Number;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Number::Int(l), Number::Int(r)) => Number::Int(l + r),
            (Number::Float(l), Number::Float(r)) => Number::Float(l + r),
            (Number::Int(l), Number::Float(r)) => Number::Float(l as f64 + r),
            (Number::Float(l), Number::Int(r)) => Number::Float(l + r as f64),
        }
    }
}

impl<'a, 'b> ops::Add<&'b Number> for &'a Number {
    type Output = Number;

    fn add(self, rhs: &'b Number) -> Self::Output {
        match (self, rhs) {
            (Number::Int(l), Number::Int(r)) => Number::Int(l + r),
            (Number::Float(l), Number::Float(r)) => Number::Float(l + r),
            (Number::Int(l), Number::Float(r)) => Number::Float(*l as f64 + r),
            (Number::Float(l), Number::Int(r)) => Number::Float(l + *r as f64),
        }
    }
}

impl ops::Sub for Number {
    type Output = Number;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Number::Int(l), Number::Int(r)) => Number::Int(l - r),
            (Number::Float(l), Number::Float(r)) => Number::Float(l - r),
            (Number::Int(l), Number::Float(r)) => Number::Float(l as f64 - r),
            (Number::Float(l), Number::Int(r)) => Number::Float(l - r as f64),
        }
    }
}

impl<'a, 'b> ops::Sub<&'a Number> for &'a Number {
    type Output = Number;

    fn sub(self, rhs: &'a Number) -> Self::Output {
        match (self, rhs) {
            (Number::Int(l), Number::Int(r)) => Number::Int(l - r),
            (Number::Float(l), Number::Float(r)) => Number::Float(l - r),
            (Number::Int(l), Number::Float(r)) => Number::Float(*l as f64 - r),
            (Number::Float(l), Number::Int(r)) => Number::Float(l - *r as f64),
        }
    }
}

impl ops::Mul for Number {
    type Output = Number;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Number::Int(l), Number::Int(r)) => Number::Int(l * r),
            (Number::Float(l), Number::Float(r)) => Number::Float(l * r),
            (Number::Int(l), Number::Float(r)) => Number::Float(l as f64 * r),
            (Number::Float(l), Number::Int(r)) => Number::Float(l * r as f64),
        }
    }
}

impl<'a, 'b> ops::Mul<&'a Number> for &'a Number {
    type Output = Number;

    fn mul(self, rhs: &'a Number) -> Self::Output {
        match (self, rhs) {
            (Number::Int(l), Number::Int(r)) => Number::Int(l * r),
            (Number::Float(l), Number::Float(r)) => Number::Float(l * r),
            (Number::Int(l), Number::Float(r)) => Number::Float(*l as f64 * r),
            (Number::Float(l), Number::Int(r)) => Number::Float(l * *r as f64),
        }
    }
}

impl ops::Div for Number {
    type Output = Number;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Number::Int(l), Number::Int(r)) => Number::Int(l / r),
            (Number::Float(l), Number::Float(r)) => Number::Float(l / r),
            (Number::Int(l), Number::Float(r)) => Number::Float(l as f64 / r),
            (Number::Float(l), Number::Int(r)) => Number::Float(l / r as f64),
        }
    }
}

impl<'a, 'b> ops::Div<&'a Number> for &'a Number {
    type Output = Number;

    fn div(self, rhs: &'a Number) -> Self::Output {
        match (self, rhs) {
            (Number::Int(l), Number::Int(r)) => Number::Int(l / r),
            (Number::Float(l), Number::Float(r)) => Number::Float(l / r),
            (Number::Int(l), Number::Float(r)) => Number::Float(*l as f64 / r),
            (Number::Float(l), Number::Int(r)) => Number::Float(l / *r as f64),
        }
    }
}

impl ops::Neg for Number {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Number::Int(n) => Number::Int(-n),
            Number::Float(n) => Number::Float(-n),
        }
    }
}

impl Number {
    pub fn try_neg(self) -> Result<Self, Error> {
        Ok(match self {
            Number::Int(v) => Number::Int(
                v.checked_neg()
                    .ok_or_else(|| Error::InvalidNegative(v.into()))?,
            ),
            Number::Float(v) => Number::Float(-v),
        })
    }
}

impl Number {
    pub fn to_usize(&self) -> usize {
        match self {
            Number::Int(i) => *i as usize,
            Number::Float(i) => *i as usize,
        }
    }
}
