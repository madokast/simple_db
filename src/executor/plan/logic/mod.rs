use crate::executor::data::schema::Schema;

/// 逻辑计划

pub trait LogicNode {
    fn schema(&self) -> &Schema;
    fn children(&self) -> &[Box<dyn LogicNode>];
    fn to_string(&self) -> String;
}
