use std::fmt::{Debug, Display};

use super::column::Column;

/// Scheme 实际就是表的元信息
/// 一个表包含多个列/字段，注意是有序的
#[derive(Clone)]
pub struct Schema {
    pub name: Box<str>, // 表名，或者中间表标识名
    pub columns: Box<[Column]>,
}

impl Schema {
    pub fn contains_column_name(&self, name: &str) -> bool {
        for column in self.columns.iter() {
            if column.name.as_ref() == name {
                return true;
            }
        }
        false
    }
}

impl Display for Schema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Debug for Schema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)?;

        for (index, column) in self.columns.iter().enumerate() {
            if index == 0 {
                write!(f, "(")?;
            } else {
                write!(f, ", ")?;
            }
            write!(f, "{:?}", column)?;
        }

        write!(f, ")")
    }
}
