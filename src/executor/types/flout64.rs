use std::{fmt::Display, ops::Add};

use super::int32::Int32;

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Float64 {
    value: f64,
}

impl Float64 {
    pub fn new(value: f64) -> Self {
        Self { value }
    }

    pub fn to_i32(&self) -> Int32 {
        Int32::new(self.value as i32)
    }

    pub fn to_string(&self) -> String {
        self.value.to_string()
    }
}

impl Add for Float64 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value + rhs.value,
        }
    }
}

impl From<f64> for Float64 {
    fn from(value: f64) -> Self {
        Self { value }
    }
}

impl Display for Float64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}
