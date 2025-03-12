/// 物理计划

pub trait PhysicalPlan {
    fn schema(&self) -> &Schema;
    fn children(&self) -> &[PhysicalPlan];
    fn to_string(&self) -> String;
}
