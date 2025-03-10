use crate::sql::tokenizer::str_scanner::TokenLocation;

pub trait WithLocation {
    fn location(&self) -> &Location;
}
// 位置信息
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Location {
    pub line_number: usize,
    pub column_number: usize,
    pub offset: usize,
}

impl WithLocation for Location {
    fn location(&self) -> &Location {
        self
    }
}

impl Location {
    pub fn new(location: &TokenLocation) -> Self {
        Self {
            line_number: location.line_number,
            column_number: location.column_number,
            offset: location.offset,
        }
    }
}
