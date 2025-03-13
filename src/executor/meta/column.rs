use std::fmt::{Debug, Display};

use crate::executor::types::DataType;

/// 列（字段）元信息
#[derive(Clone)]
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

impl Debug for Column {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let nullable = if self.nullable {
            "NULLABLE"
        } else {
            "NOT NULL"
        };
        write!(f, "{} {} {}", self.name, self.data_type, nullable)
    }
}
