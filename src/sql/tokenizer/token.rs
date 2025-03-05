use std::fmt::Display;

use super::str_scanner::TokenLocation;

#[derive(
    Clone,
    Copy,
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

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Token {
    Keyword(Keyword),
    Identifier(String), // 表名、列名
    StringLiteral(String),
    IntegerLiteral(u16, Option<u64>), // 前导零数目 + 数字
    Equal,                            // =
    NotEqual,                         // <> or !=
    LessThan,                         // <
    GreaterThan,                      // >
    LessThanOrEqual,                  // <=
    GreaterThanOrEqual,               // >=
    Plus,                             // +
    Minus,                            // -
    Multiply,                         // *
    Divide,                           // /
    LeftParenthesis,                  // (
    RightParenthesis,                 // )
    Comma,                            // 逗号
    Semicolon,                        // 分号
    Period,                           // .
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Keyword(keyword) => write!(f, "{}", keyword),
            Token::Identifier(ident) => write!(f, "{}", ident),
            Token::StringLiteral(s) => write!(f, "'{}'", s),
            Token::IntegerLiteral(zeros, num) => match num {
                Some(n) => write!(f, "{:0>width$}{}", "", n, width = *zeros as usize),
                None => write!(f, "{:0>width$}", "", width = *zeros as usize),
            },
            Token::Equal => write!(f, "="),
            Token::NotEqual => write!(f, "<>"),
            Token::LessThan => write!(f, "<"),
            Token::GreaterThan => write!(f, ">"),
            Token::LessThanOrEqual => write!(f, "<="),
            Token::GreaterThanOrEqual => write!(f, ">="),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Multiply => write!(f, "*"),
            Token::Divide => write!(f, "/"),
            Token::LeftParenthesis => write!(f, "("),
            Token::RightParenthesis => write!(f, ")"),
            Token::Comma => write!(f, ","),
            Token::Semicolon => write!(f, ";"),
            Token::Period => write!(f, "."),
        }
    }
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

impl Display for ParsedTokens {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (index, token) in self.tokens().iter().enumerate() {
            // 有些 token 前需要加上空格
            if index > 0 {
                match token.token {
                    Token::Keyword(_)
                    | Token::Identifier(_)
                    | Token::StringLiteral(_)
                    | Token::IntegerLiteral(_, _) => f.write_str(" ")?,
                    Token::Equal
                    | Token::NotEqual
                    | Token::LessThan
                    | Token::GreaterThan
                    | Token::LessThanOrEqual
                    | Token::GreaterThanOrEqual
                    | Token::Plus
                    | Token::Minus
                    | Token::Multiply
                    | Token::Divide
                    | Token::LeftParenthesis
                    | Token::RightParenthesis
                    | Token::Period => f.write_str(" ")?,
                    Token::Comma | Token::Semicolon => {}
                };
            }

            f.write_fmt(format_args!("{}", token.token))?;
        }
        Ok(())
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

#[cfg(test)]
mod test {
    use crate::sql::tokenizer::token::Keyword;

    use super::Token;

    #[test]
    fn token_display_kw() {
        assert_eq!(format!("{}", Token::Keyword(Keyword::SELECT)), "SELECT");
    }

    #[test]
    fn token_display_id() {
        assert_eq!(
            format!("{}", Token::Identifier("abc123".to_string())),
            "abc123"
        );
    }

    #[test]
    fn token_display_number() {
        assert_eq!(format!("{}", Token::IntegerLiteral(0, Some(123))), "123");
    }

    #[test]
    fn token_display_number2() {
        assert_eq!(format!("{}", Token::IntegerLiteral(3, Some(123))), "000123");
    }

    #[test]
    fn token_display_number3() {
        assert_eq!(format!("{}", Token::IntegerLiteral(4, None)), "0000");
    }
}
