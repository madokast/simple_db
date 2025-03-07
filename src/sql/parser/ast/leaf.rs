use crate::sql::tokenizer::str_scanner::TokenLocation;

// 叶子节点附带信息
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Leaf {
    pub line_number: usize,
    pub column_number: usize,
    pub offset: usize,
}

impl Leaf {
    pub fn new(location: &TokenLocation) ->Self {
        Self {
            line_number: location.line_number,
            column_number: location.column_number,
            offset: location.offset,
        }
    }
}