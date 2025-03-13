use crate::executor::{
    error::ExecuteError,
    meta::{row::Row, rows::Rows, schema::Schema},
};

pub mod scan;

/// 物理计划
pub trait PhysicalPlan {
    fn children(&self) -> &[&dyn PhysicalPlan];
    fn schema(&self) -> &Schema;
    fn to_string(&self) -> String;

    fn open(&mut self) -> Result<(), ExecuteError>;
    fn next(&mut self) -> Result<Option<Box<dyn Row>>, ExecuteError>;
    fn batch(&mut self) -> Result<Option<Box<dyn Rows>>, ExecuteError>;
    fn close(&mut self) -> Result<(), ExecuteError>;
}
