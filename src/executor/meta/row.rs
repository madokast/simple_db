use std::fmt::Debug;

use crate::executor::types::{
    flout64::Float64, int32::Int32, varchar::Varchar, DataType, OwnValue,
};

use super::schema::Schema;

/// Row 行接口
pub trait Row: Debug {
    fn is_null(&self, index: usize) -> bool;
    fn get_int32(&self, index: usize) -> Int32;
    fn get_float64(&self, index: usize) -> Float64;
    fn get_varchar<'a>(&'a self, index: usize) -> Varchar<'a>;
    fn get_string<'a>(&'a self, index: usize) -> &'a str;

    fn get(&self, index: usize) -> &OwnValue;

    fn to_string<'a>(&'a self, schema: &Schema) -> String {
        let mut buf: String = "[".to_string();
        for (index, column) in schema.columns.iter().enumerate() {
            if index > 0 {
                buf.push_str(", ");
            }
            if self.is_null(index) {
                buf.push_str("NULL");
                continue;
            }
            match column.data_type {
                DataType::Int32 => {
                    buf.push_str(&self.get_int32(index).to_string());
                }
                DataType::Float64 => {
                    buf.push_str(&self.get_float64(index).to_string());
                }
                DataType::Varchar(_) => {
                    buf.push('"');
                    buf.push_str(&self.get_varchar(index).to_string());
                    buf.push('"');
                }
                DataType::String => {
                    buf.push('"');
                    buf.push_str(&self.get_string(index).to_string());
                    buf.push('"');
                }
            }
        }
        buf.push(']');
        buf
    }
}

#[derive(Debug, Clone)]
pub struct SimpleMemoryRow {
    values: Box<[OwnValue]>,
}

impl SimpleMemoryRow {
    pub fn new(values: Vec<OwnValue>) -> Self {
        Self {
            values: values.into_boxed_slice(),
        }
    }
}

impl Row for SimpleMemoryRow {
    fn is_null(&self, index: usize) -> bool {
        match &self.values[index] {
            OwnValue::Null => true,
            _ => false,
        }
    }

    fn get_int32(&self, index: usize) -> Int32 {
        match &self.values[index] {
            OwnValue::Int32(v) => *v,
            _ => panic!("type mismatch"),
        }
    }

    fn get_float64(&self, index: usize) -> Float64 {
        match &self.values[index] {
            OwnValue::Float64(v) => *v,
            _ => panic!("type mismatch"),
        }
    }

    fn get_varchar(&self, index: usize) -> Varchar<'_> {
        match &self.values[index] {
            OwnValue::String(s) => Varchar::ref_string(s),
            _ => panic!("type mismatch"),
        }
    }

    fn get_string(&self, index: usize) -> &str {
        match &self.values[index] {
            OwnValue::String(s) => s,
            _ => panic!("type mismatch"),
        }
    }

    fn get(&self, index: usize) -> &OwnValue {
        &self.values[index]
    }
}
