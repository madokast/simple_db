use std::{fmt::Display, ops::Add};

use super::flout64::Float64;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Int32 {
    value: i32,
}

impl Int32 {
    pub fn new(value: i32) -> Self {
        Self { value }
    }

    pub fn to_float64(&self) -> Float64 {
        Float64::new(self.value as f64)
    }

    pub fn to_string(&self) -> String {
        self.value.to_string()
    }
}

impl Add for Int32 {
    type Output = Int32;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value + rhs.value,
        }
    }
}

impl From<i32> for Int32 {
    fn from(value: i32) -> Self {
        Self { value }
    }
}

impl Display for Int32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}
