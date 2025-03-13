use crate::executor::meta::{data_source::DataSource, schema::Schema};

use super::LogicPlan;

#[derive(Debug)]
pub struct Scan<'a> {
    pub data_source: &'a dyn DataSource,
    pub projection: Box<[Box<str>]>,
    pub schema: Schema,
}

impl<'a> Scan<'a> {
    pub fn new(
        data_source: &'a dyn DataSource,
        projection: Box<[Box<str>]>,
        schema: Schema,
    ) -> Self {
        #[cfg(debug_assertions)]
        {
            assert!(projection.len() > 0, "projection must not be empty");
            for col in &projection {
                assert!(
                    schema.contains_column_name(col.as_ref()),
                    "column {} not found in schema",
                    col
                );
            }
        }
        Self {
            data_source,
            projection,
            schema,
        }
    }
}

impl<'a> LogicPlan for Scan<'a> {
    fn children(&self) -> &[&dyn LogicPlan] {
        &[]
    }

    fn schema(&self) -> &Schema {
        &self.schema
    }

    fn to_string(&self) -> String {
        format!("{}", self.schema)
    }
}
