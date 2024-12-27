use core::fmt;

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub enum Direction {
    In,
    Out,
    Both,
}

impl Direction {
    pub fn is_in(&self) -> bool {
        matches!(self, Direction::In)
    }

    pub fn is_out(&self) -> bool {
        matches!(self, Direction::Out)
    }

    pub fn is_both(&self) -> bool {
        matches!(self, Direction::Both)
    }
}

impl Default for Direction {
    fn default() -> Self {
        Self::Both
    }
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Direction::In => write!(f, "->"),
            Direction::Out => write!(f, "<-"),
            Direction::Both => write!(f, "<->"),
        }
    }
}
