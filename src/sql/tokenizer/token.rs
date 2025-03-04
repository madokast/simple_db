#[derive(Debug, PartialEq, Eq, Hash, strum_macros::Display, strum_macros::EnumCount, strum_macros::EnumIter)]
pub enum Keyword {
    SELECT,
    FROM,
    WHERE
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Token {
    Keyword(Keyword),
    Identifier(String), // 表名、列名
    StringLiteral(String),
    IntegerLiteral(i64),
    Comma, // 逗号
    Semicolon, // 分号
    Asterisk, // 星号
}

// static KEYWORDS: HashMap<String, Keyword> = {
//     let mut m = HashMap::new();
//     m
// };

