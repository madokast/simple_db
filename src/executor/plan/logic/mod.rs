use crate::executor::meta::schema::Schema;

/// 表扫描逻辑计划
pub mod scan;

/// 逻辑计划
pub trait LogicPlan {
    fn children(&self) -> &[&dyn LogicPlan];
    fn schema(&self) -> &Schema;
    fn to_string(&self) -> String;
}
