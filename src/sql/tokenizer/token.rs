use super::str_scanner::TokenLocation;

#[derive(
    Debug,
    PartialEq,
    Eq,
    Hash,
    strum_macros::Display,
    strum_macros::EnumCount,
    strum_macros::EnumIter,
)]
pub enum Keyword {
    SELECT,
    FROM,
    WHERE,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Token {
    Keyword(Keyword),
    Identifier(String), // 表名、列名
    StringLiteral(String),
    IntegerLiteral(i64),
    Comma,     // 逗号
    Semicolon, // 分号
    Asterisk,  // 星号
}

#[derive(Debug)]
pub struct ParsedTokens {
    tokens: Vec<ParsedToken>,
    pub raw_sql: String,
}

impl ParsedTokens {
    pub fn new(tokens: Vec<ParsedToken>, raw_sql: String) -> Self {
        Self { tokens, raw_sql }
    }

    pub fn tokens(&self) -> &[ParsedToken] {
        &self.tokens
    }
}

#[derive(Debug)]
pub struct ParsedToken {
    pub token: Token,
    pub location: TokenLocation,
}

impl ParsedToken {
    pub fn new(token: Token, location: TokenLocation) -> Self {
        Self { token, location }
    }
}
