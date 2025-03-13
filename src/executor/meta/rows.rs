use crate::executor::types::{flout64::Float64, int32::Int32, varchar::Varchar};

pub trait Rows {
    fn next(&mut self) -> bool;

    fn is_null(&self, index: usize) -> bool;
    fn get_int32(&self, index: usize) -> Int32;
    fn get_float64(&self, index: usize) -> Float64;
    fn get_varchar<'a>(&'a self, index: usize) -> Varchar<'a>;
    fn get_string<'a>(&'a self, index: usize) -> &'a str;
}
