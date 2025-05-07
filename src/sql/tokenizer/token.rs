use std::{collections::HashMap, fmt::Display, rc::Rc, sync::LazyLock};

use super::str_scanner::TokenLocation;

/// SQL 关键字
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Keyword {
    SELECT,
    FROM,
    WHERE,
    GROUP,
    BY,
    ORDER,
    LIMIT,
    OFFSET,
    AS,
    DESC,
    ASC,
    CREATE,
    TABLE,
    IS,
    NULL,
    AND,
    OR,
    NOT,
    HAVING,
} // 注意同步更新 ALL_KEY_WORDS

use Keyword::*;

/// 关键字数组
const ALL_KEY_WORDS: [Keyword; 19] = [
    SELECT, FROM, WHERE, GROUP, BY, ORDER, LIMIT, OFFSET, AS, DESC, ASC, CREATE, TABLE, IS, NULL,
    AND, OR, NOT, HAVING,
];

/// 关键字映射。string -> keyword
static KEYWORD_MAP: LazyLock<HashMap<&'static str, Keyword>> = LazyLock::new(|| {
    let mut key_words: HashMap<&'static str, Keyword> = HashMap::new();
    Keyword::all().iter().for_each(|kw| {
        key_words.insert(kw.to_str(), *kw);
    });
    key_words
});

/// 关键字最大长度，词法分析时，如果符号长度超过该值，则认为不是关键字，直接跳过关键字映射判断
static KEYWORD_MAX_LENGTH: LazyLock<usize> = LazyLock::new(|| {
    Keyword::all()
        .iter()
        .map(|kw| kw.to_str().len())
        .max()
        .unwrap()
});

impl Keyword {
    pub fn all() -> &'static [Keyword] {
        &ALL_KEY_WORDS
    }

    /// 获取关键字的映射，不应频繁调用
    pub fn map() -> &'static HashMap<&'static str, Keyword> {
        // LazyLock::force(&KEY_WORD_MAP)
        &*KEYWORD_MAP
    }

    /// 获取关键字的最大长度，不应频繁调用
    pub fn max_length() -> usize {
        // *LazyLock::force(&KEYWORD_MAX_LENGTH)
        *&*KEYWORD_MAX_LENGTH
    }

    /// 关键字转换到字符串
    pub fn to_str(&self) -> &'static str {
        match self {
            SELECT => "SELECT",
            FROM => "FROM",
            WHERE => "WHERE",
            GROUP => "GROUP",
            BY => "BY",
            ORDER => "ORDER",
            LIMIT => "LIMIT",
            OFFSET => "OFFSET",
            AS => "AS",
            DESC => "DESC",
            ASC => "ASC",
            CREATE => "CREATE",
            TABLE => "TABLE",
            IS => "IS",
            NULL => "NULL",
            AND => "AND",
            OR => "OR",
            NOT => "NOT",
            HAVING => "HAVING",
        }
    }
}

impl Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

/// SQL Token 类型
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Token {
    Keyword(Keyword),                 // 关键字
    Identifier(Rc<str>),              // 表名、列名。使用 Rc<str> 类型，引用原始 SQL 字符串
    StringLiteral(Rc<str>),           // 字符串字面量。使用 Rc<str> 类型，引用原始 SQL 字符串
    IntegerLiteral(u16, Option<u64>), // 前导零数目 + 数字。此时整数和小数没有区分，小数表示为 数字 + 句号 + 数字
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
            Token::Keyword(keyword) => f.write_str(keyword.to_str()),
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

/// 解析后的 SQL Token 序列
#[derive(Debug, Clone)]
pub struct ParsedTokens<'a> {
    pub tokens: Box<[ParsedToken]>, // 解析后的 Token 序列。Box 表示对数组的只读所有
    pub raw_sql: &'a str,          // 原始 SQL 字符串。
}

impl<'a> ParsedTokens<'a> {
    pub fn new(tokens: Vec<ParsedToken>, raw_sql: &'a str) -> Self {
        Self {
            tokens: tokens.into_boxed_slice(),
            raw_sql: raw_sql,
        }
    }

    pub fn tokens(&self) -> &[ParsedToken] {
        &self.tokens
    }
}

impl<'a> Display for ParsedTokens<'a> {
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

#[derive(Debug, Clone)]
pub struct ParsedToken {
    pub token: Token,
    pub location: TokenLocation,
}

impl ParsedToken {
    pub fn new(token: Token, location: TokenLocation) -> Self {
        Self { token, location }
    }
}

impl Display for ParsedToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.token)
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
        assert_eq!(format!("{}", Token::Identifier("abc123".into())), "abc123");
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
