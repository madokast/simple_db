use std::fmt::{Debug, Display};

use super::{
    row::{Row, SimpleMemoryRow},
    schema::Schema,
};

pub trait DataSource {
    fn schema(&self) -> &Schema;
    fn read(&self) -> impl Iterator<Item = &impl Row>;
}

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
    fn schema(&self) -> &Schema {
        &self.schema
    }

    #[allow(refining_impl_trait)]
    fn read(&self) -> impl Iterator<Item = &SimpleMemoryRow> {
        self.rows.iter()
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

mod test {
    use crate::executor::{
        meta::{
            column::Column,
            data_source::{DataSource, SimpleMemoryDataSource},
            row::{Row, SimpleMemoryRow},
            schema::Schema,
        },
        types::{int32::Int32, DataType, OwnValue},
    };

    #[test]
    fn test_data_source() {
        let column_name: Column = Column {
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
