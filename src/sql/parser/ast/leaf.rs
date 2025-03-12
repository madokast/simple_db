use crate::sql::tokenizer::str_scanner::TokenLocation;

/// AST 所有节点都是 WithLocation
/// 表示节点在原 SQL 语句中的位置信息
/// 用于在错误信息中定位
pub trait WithLocation {
    fn location(&self) -> &Location;

    fn locate(&self, raw_sql: &str) -> String {
        let loc = self.location();
        const SKIP_BACKWARD: usize = 16;
        let skip: usize = {
            if loc.offset > SKIP_BACKWARD {
                loc.offset - SKIP_BACKWARD
            } else {
                0
            }
        };
        let near: String = raw_sql.chars().skip(skip).take(SKIP_BACKWARD * 2).collect();
        format!(
            "Ln {}, Col {} near \"{}\"",
            loc.line_number, loc.column_number, near
        )
    }
}
// Location 位置信息具体类
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
