use super::leaf::Leaf;

pub struct Literal {
    pub value: Value,
    pub leaf: Leaf
}

pub enum Value {
    String(String),
    Integer(u64),
    Float(f64),
}


