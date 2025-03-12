use super::data_type::DataType;

/// 列（字段）元信息
pub struct Column {
    pub name: Box<str>,
    pub data_type: DataType,
    pub nullable: bool,
}
