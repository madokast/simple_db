use std::fmt::{Debug, Display};

use crate::executor::types::DataType;

/// 列（字段）元信息
#[derive(Debug, Clone)]
pub struct Column {
    pub name: Box<str>,
    pub data_type: DataType,
    pub nullable: bool,
}

impl Display for Column {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
