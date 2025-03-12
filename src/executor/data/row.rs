pub trait Row {
    fn len(&self) -> usize;
    fn is_null(&self, index: usize) -> bool;
    fn get_i64(&self, index: usize) -> i64;
    fn get_f64(&self, index: usize) -> f64;
    fn get_bool(&self, index: usize) -> bool;
    fn get_string(&self, index: usize) -> &str;
}

#[derive(Debug)]
pub enum MemoryValue {
    I64(i64),
    F64(f64),
    Bool(bool),
    String(Box<str>),
    Null,
}

#[derive(Debug)]
pub struct MemoryRow {
    pub values: Vec<MemoryValue>,
}

impl Row for MemoryRow {
    fn len(&self) -> usize {
        self.values.len()
    }

    fn is_null(&self, index: usize) -> bool {
        match &self.values[index] {
            MemoryValue::Null => true,
            _ => false,
        }
    }

    fn get_i64(&self, index: usize) -> i64 {
        match &self.values[index] {
            MemoryValue::I64(v) => *v,
            _ => panic!("value at index {}({:?}) is not i64,", index, &self.values[index]),
        }
    }

    fn get_f64(&self, index: usize) -> f64 {
        match &self.values[index] {
            MemoryValue::F64(v) => *v,
            _ => panic!("value at index {}({:?}) is not f64,", index, &self.values[index]),
        }
    }

    fn get_bool(&self, index: usize) -> bool {
        match &self.values[index] {
            MemoryValue::Bool(v) => *v,
            _ => panic!("value at index {}({:?}) is not bool,", index, &self.values[index]),
        }
    }

    fn get_string(&self, index: usize) -> &str {
        match &self.values[index] {
            MemoryValue::String(v) => v,
            _ => panic!("value at index {}({:?}) is not string,", index, &self.values[index]),
        }
    }
}
