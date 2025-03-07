use super::leaf::Leaf;

#[derive(Debug, PartialEq, Clone)]
pub struct Literal {
    pub value: Value,
    pub leaf: Leaf,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    String(String),
    Integer(u64),
    Float(f64),
}
