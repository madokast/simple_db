/// 数据类型的大小
pub enum DataSize {
    Bit8,
    Bit16,
    Bit32,
    Bit64,
}

pub enum DataType {
    Integer(DataSize),
    Float(DataSize),
    String,
    Boolean,
}
