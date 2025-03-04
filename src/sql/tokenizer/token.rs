
pub enum Keyword {
    SELECT,
    FROM,
    WHERE
}


pub enum Token {
    Keyword(Keyword),
    Identifier(String), // 表名、列名
    StringLiteral(String),
    IntegerLiteral(i64),
    FloatLiteral(f64),
    Comma, // 逗号
    Semicolon, // 分号
    Asterisk, // 星号
}

