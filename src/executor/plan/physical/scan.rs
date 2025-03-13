use crate::executor::{
    error::ExecuteError,
    meta::{
        data_source::DataSource,
        row::{Row, SimpleMemoryRow},
        rows::Rows,
        schema::Schema,
    },
};

use super::PhysicalPlan;

pub struct SeqScan<'a> {
    pub data_source: &'a dyn DataSource,
    pub projection: Box<[u16]>,
    pub schema: Schema,

    iter: Option<Box<dyn Iterator<Item = &'a dyn Row> + 'a>>,
}

impl<'a> PhysicalPlan for SeqScan<'a> {
    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn children(&self) -> &[&dyn PhysicalPlan] {
        &[]
    }

    fn to_string(&self) -> String {
        format!("{}", self.schema)
    }

    fn open(&mut self) -> Result<(), ExecuteError> {
        debug_assert!(self.iter.is_none());
        self.iter = Some(self.data_source.read());
        Ok(())
    }

    fn next(&mut self) -> Result<Option<Box<dyn Row>>, ExecuteError> {
        let iter: &mut Box<dyn Iterator<Item = &dyn Row>> = self.iter.as_mut().unwrap();
        let row: Option<&dyn Row> = iter.next();
        match row {
            Some(row) => {
                let mut values = Vec::with_capacity(self.projection.len());
                for index in self.projection.iter() {
                    values.push(row.get(*index as usize).clone());
                }
                let row = SimpleMemoryRow::new(values);
                Ok(Some(Box::new(row)))
            }
            None => Ok(None),
        }
    }

    fn batch(&mut self) -> Result<Option<Box<dyn Rows>>, ExecuteError> {
        panic!("not implemented")
    }

    fn close(&mut self) -> Result<(), ExecuteError> {
        // self.iter = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::executor::{
        meta::{
            column::Column,
            data_source::{DataSource, SimpleMemoryDataSource},
            row::SimpleMemoryRow,
            schema::Schema,
        },
        plan::physical::{scan::SeqScan, PhysicalPlan},
        types::{int32::Int32, DataType, OwnValue},
    };

    #[test]
    fn test_scan() {
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

        let mut scan: Box<SeqScan<'_>> = Box::new(SeqScan {
            data_source: &source,
            projection: vec![1].into_boxed_slice(),
            schema: source.schema().clone(),
            iter: None,
        });

        scan.open().unwrap();

        while let Some(row) = scan.next().unwrap() {
            println!("row = {:?}", row);
        }

        scan.close().unwrap();
    }
}
