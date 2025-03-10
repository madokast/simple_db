use std::{fmt::Display, rc::Rc};

use super::leaf::{Location, WithLocation};

#[derive(Debug, PartialEq, Clone)]
pub struct Literal {
    pub value: Value,
    pub leaf: Location,
}

impl WithLocation for Literal  {
    fn location(&self) -> &Location {
        &self.leaf
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    String(Rc<str>),
    Integer(u64),
    Float(f64),
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.value {
            Value::String(s) => write!(f, "'{}'", s),
            Value::Integer(n) => write!(f, "{}", n),
            Value::Float(n) => write!(f, "{}", n),
        }
    }
}
