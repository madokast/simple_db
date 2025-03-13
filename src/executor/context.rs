use std::collections::HashMap;

use super::meta::data_source::DataSource;

pub trait Context {
    fn find_table<'a>(&'a self, name: &str) -> Option<&'a dyn DataSource>;
}

pub struct SimpleMemoryContext {
    tables: HashMap<Box<str>, Box<dyn DataSource>>,
}

impl Context for SimpleMemoryContext {
    fn find_table<'a>(&'a self, name: &str) -> Option<&'a dyn DataSource> {
        self.tables.get(name).map(|v| v.as_ref())
    }
}
