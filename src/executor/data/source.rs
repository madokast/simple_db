use crate::executor::data::{data_type::DataType, row::MemoryValue};

use super::{row::{MemoryRow, Row}, schema::Schema};

pub trait DataSource {
    fn schema(&self) -> &Schema;
    fn row_iterator(&self) -> impl RowIterator;
}

pub trait RowIterator {
    fn next(&mut self) -> Option<&dyn Row>;
    fn close(&mut self);
}

pub struct MemoryTable {
    schema: Schema,
    rows: Vec<MemoryRow>,
}

pub struct MemoryTableIterator {
    rows: std::slice::Iter<'static, MemoryRow>,
}

impl RowIterator for MemoryTableIterator {
    fn next(&mut self) -> Option<&dyn Row> {
        self.rows.next().map(|row| row as &dyn Row)
    }

    fn close(&mut self) {
        // do nothing
    }
}

impl DataSource for MemoryTable {
    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn row_iterator(&self) -> impl RowIterator {
        MemoryTableIterator {
            rows: self.rows.iter(),
        }
    }
}

impl MemoryTable {
    pub fn new(schema: Schema) -> Self {
        MemoryTable {
            schema,
            rows: vec![],
        }
    }

    pub fn add_row(&mut self, row: MemoryRow) {
        assert_eq!(self.schema.columns.len(), row.len());
        for (column, val) in self.schema.column_iter().zip(row.values.iter()) {
            match column.data_type {
                DataType::Integer(_) => {
                    if let MemoryValue::I64(_) = val {
                    } else {
                        panic!("expect i64, got {:?}", val);
                    }
                }
                DataType::Float(_) => {
                    if let MemoryValue::F64(_) = val {
                    } else {
                        panic!("expect f64, got {:?}", val);
                    }
                }
                DataType::Boolean => {
                    if let MemoryValue::Bool(_) = val {
                    } else {
                        panic!("expect bool, got {:?}", val);
                    }
                }
                DataType::String => {
                    if let MemoryValue::String(_) = val {
                    } else {
                        panic!("expect string, got {:?}", val);
                    }
                }
            }
        }
        self.rows.push(row);
    }
}
