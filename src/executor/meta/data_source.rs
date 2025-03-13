use std::fmt::{Debug, Display};

use super::{
    row::{Row, SimpleMemoryRow},
    rows::Rows,
    schema::Schema,
};

pub trait DataSource: Debug {
    fn name(&self) -> String;
    fn schema(&self) -> &Schema;
    fn read<'a>(&'a self) -> Box<dyn Iterator<Item = &'a dyn Row> + 'a>;
    fn batch_read<'a>(&'a self) -> Box<dyn Iterator<Item = &'a dyn Rows> + 'a>;
}

#[derive(Clone)]
pub struct SimpleMemoryDataSource {
    schema: Schema,
    rows: Vec<SimpleMemoryRow>,
}

impl SimpleMemoryDataSource {
    pub fn new(schema: Schema) -> Self {
        Self {
            schema,
            rows: Vec::new(),
        }
    }

    pub fn push_row(&mut self, row: SimpleMemoryRow) {
        self.rows.push(row);
    }
}

impl DataSource for SimpleMemoryDataSource {
    fn name(&self) -> String {
        self.schema.name.as_ref().to_string()
    }

    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn read<'a>(&'a self) -> Box<dyn Iterator<Item = &'a dyn Row> + 'a> {
        Box::new(self.rows.iter().map(|r| r as &dyn Row))
    }

    fn batch_read<'a>(&'a self) -> Box<dyn Iterator<Item = &'a dyn Rows> + 'a> {
        panic!("not implemented")
    }
}

impl Display for SimpleMemoryDataSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.schema)
    }
}

impl Debug for SimpleMemoryDataSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.schema)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::executor::{
        meta::{column::Column, schema::Schema},
        types::{int32::Int32, DataType, OwnValue},
    };

    #[test]
    fn test_data_source() {
        let column_name = Column {
            name: "name".into(),
            data_type: DataType::Varchar(32),
            nullable: true,
        };
        println!("column_name = {:?}", column_name);

        let column_age: Column = Column {
            name: "age".into(),
            data_type: DataType::Int32,
            nullable: true,
        };
        println!("column_age = {:?}", column_age);

        let schema: Schema = Schema {
            name: "stu".into(),
            columns: vec![column_name, column_age].into_boxed_slice(),
        };
        println!("schema = {:?}", schema);

        let mut source: SimpleMemoryDataSource = SimpleMemoryDataSource::new(schema);
        source.push_row(SimpleMemoryRow::new(vec![
            OwnValue::String("张三".into()),
            OwnValue::Int32(Int32::new(18)),
        ]));
        source.push_row(SimpleMemoryRow::new(vec![
            OwnValue::Null,
            OwnValue::Int32(Int32::new(20)),
        ]));
        source.push_row(SimpleMemoryRow::new(vec![
            OwnValue::String("王五".into()),
            OwnValue::Null,
        ]));
        println!("source = {:?}", source);

        for row in source.read() {
            println!("row = {}", row.to_string(source.schema()));
        }
    }
}
