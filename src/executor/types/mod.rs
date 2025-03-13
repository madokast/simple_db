use std::fmt::Display;

pub mod flout64;
pub mod int32;
pub mod varchar;

#[derive(Debug, PartialEq, Clone)]
pub enum DataType {
    Int32,
    Float64,
    Varchar(u16), // 不拥有数据
    String,       // 拥有数据，作为中间数据
}

impl Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataType::Int32 => write!(f, "Int32"),
            DataType::Float64 => write!(f, "Float64"),
            DataType::Varchar(len) => write!(f, "Varchar({})", len),
            DataType::String => write!(f, "String"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum OwnValue {
    Int32(int32::Int32),
    Float64(flout64::Float64),
    String(String),
    Null,
}
