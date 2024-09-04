#[derive(Debug, Clone, PartialEq)]
pub enum Number {
    Int(i64),
    Float(f64),
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
