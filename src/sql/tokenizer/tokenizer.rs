use std::{collections::HashMap, fmt::Arguments};

use super::{
    error::TokenizeError,
    str_scanner::TokenLocation,
    token::{Keyword, ParsedToken, ParsedTokens, Token},
};

use super::str_scanner::Scanner;

pub struct Tokenizer<'a> {
    key_words: &'static HashMap<&'static str, Keyword>,
    ley_word_max_length: usize,
    sql: &'a str,
    scanner: Scanner<'a>,
}

impl<'a> Tokenizer<'a> {
    pub fn new(sql: &'a str) -> Self {
        Self {
            key_words: Keyword::map(),
            ley_word_max_length: Keyword::max_length(),
            sql: sql,
            scanner: Scanner::new(sql),
        }
    }

    /// tokenize the sql
    pub fn tokenize(&mut self) -> Result<ParsedTokens, TokenizeError> {
        let mut tokens: Vec<ParsedToken> = vec![];

        // 循环获取下一个 token
        loop {
            match self.next_token() {
                Ok(token) => match token {
                    Some(t) => tokens.push(t),
                    None => break,
                },
                Err(e) => return Err(e),
            }
        }

        return Ok(ParsedTokens::new(tokens, self.sql));
    }

    /// read next_token from scanner
    fn next_token(&mut self) -> Result<Option<ParsedToken>, TokenizeError> {
        loop {
            match self.peek_char() {
                Some(first) => match first {
                    'a'..='z' | 'A'..='Z' => return self.next_id_kw(first),
                    '\'' => return self.next_string(),
                    '0'..='9' => return self.next_number(first),
                    ' ' | '\r' | '\n' => self.next_char(),
                    '=' => return self.token_and_next(Token::Equal),
                    ';' => return self.token_and_next(Token::Semicolon),
                    '.' => return self.token_and_next(Token::Period),
                    ',' => return self.token_and_next(Token::Comma),
                    '+' => return self.token_and_next(Token::Plus),
                    '-' => return self.token_and_next(Token::Minus),
                    '*' => return self.token_and_next(Token::Multiply),
                    '/' => return self.token_and_next(Token::Divide),
                    '(' => return self.token_and_next(Token::LeftParenthesis),
                    ')' => return self.token_and_next(Token::RightParenthesis),
                    '<' => return self.next_less(),
                    '>' => return self.next_great(),
                    '!' => return self.next_bang(),
                    _ => return self.make_error(format_args!("unknown char {}", first)),
                },
                None => return Ok(None),
            }
        }
    }

    /// Reads the next bang neq
    fn next_bang(&mut self) -> Result<Option<ParsedToken>, TokenizeError> {
        let start_location: super::str_scanner::TokenLocation = self.location();
        self.next_char();
        let token: Token = match self.peek_char() {
            Some(c) => match c {
                '=' => {
                    self.next_char();
                    Token::NotEqual
                }
                _ => {
                    return self.make_error(format_args!("unexpected char {}", c));
                }
            },
            None => {
                return self.make_error(format_args!("unexpected end of sql"));
            }
        };
        Ok(Some(ParsedToken::new(token, start_location)))
    }

    /// Read the next string literal
    fn next_string(&mut self) -> Result<Option<ParsedToken>, TokenizeError> {
        let start_location: super::str_scanner::TokenLocation = self.location();
        let mut text: String = String::new();

        self.next_char(); // consume
        while let Some(c) = self.peek_char() {
            match c {
                '\'' => {
                    // may end string but may escape
                    self.next_char();
                    if let Some(n) = self.peek_char() {
                        match n {
                            '\'' => {
                                // escape
                                // escape
                                text.push('\'');
                                self.next_char();
                            }
                            ';' | '=' | '>' | '<' | ',' | '.' => {
                                // end string
                                break;
                            }
                            ' ' | '\r' | '\n' => {
                                // end and consume
                                self.next_char();
                                break;
                            }
                            _ => {
                                return self.make_error(format_args!(
                                    "unexpected char {} after text {}",
                                    n, text
                                ));
                            }
                        }
                    }
                }
                '\r' | '\n' => {
                    // newline in string reading
                    return self.make_error(format_args!("unexpected newline in string literal"));
                }
                '\\' => {
                    // escape \
                    self.next_char();
                    match self.peek_char() {
                        Some(c) => match c {
                            '\\' => text.push('\\'),
                            '\'' => text.push('\''),
                            '\"' => text.push('\"'),
                            'n' => text.push('\n'),
                            't' => text.push('\t'),
                            'r' => text.push('\r'),
                            '0' => text.push('\0'),
                            _ => return self.make_error(format_args!("unknown escape char {}", c)),
                        },
                        None => {
                            return self
                                .make_error(format_args!("unexpected end of string literal"))
                        }
                    }
                    self.next_char();
                }
                _ => {
                    text.push(c);
                    self.next_char();
                }
            }
        }

        Ok(Some(ParsedToken::new(
            Token::StringLiteral(text.into()),
            start_location,
        )))
    }

    /// Reads the next gt, gte
    fn next_great(&mut self) -> Result<Option<ParsedToken>, TokenizeError> {
        let start_location: super::str_scanner::TokenLocation = self.location();

        self.next_char();
        let token: Token = match self.peek_char() {
            Some(c) => match c {
                '=' => {
                    self.next_char();
                    Token::GreaterThanOrEqual
                }
                _ => Token::GreaterThan,
            },
            None => {
                return self.make_error(format_args!("unexpected end of sql"));
            }
        };

        Ok(Some(ParsedToken::new(token, start_location)))
    }

    /// Reads the next lt, lte, neq
    fn next_less(&mut self) -> Result<Option<ParsedToken>, TokenizeError> {
        let start_location: super::str_scanner::TokenLocation = self.location();

        self.next_char();
        let token: Token = match self.peek_char() {
            Some(c) => match c {
                '=' => {
                    self.next_char();
                    Token::LessThanOrEqual
                }
                '>' => {
                    self.next_char();
                    Token::NotEqual
                }
                _ => Token::LessThan,
            },
            None => {
                return self.make_error(format_args!("unexpected end of sql"));
            }
        };

        Ok(Some(ParsedToken::new(token, start_location)))
    }

    /// Reads the next number
    fn next_number(&mut self, first: char) -> Result<Option<ParsedToken>, TokenizeError> {
        let start_location: super::str_scanner::TokenLocation = self.location();

        let mut leading_zero: u16 = 0;
        let mut number = None;

        let first: u64 = first as u64 - '0' as u64;
        if first == 0 {
            leading_zero += 1;
        } else {
            number = Some(first);
        }

        self.next_char(); // consume
        while let Some(c) = self.peek_char() {
            match c {
                '0' => {
                    // 如果 number 存在，则 number*=10，否则 leading_zero++
                    if let Some(num) = number {
                        number = num.checked_mul(10);
                        if number.is_none() {
                            return self.make_error(format_args!("too large number {} * 10", num));
                        }
                    } else {
                        if let Some(temp) = leading_zero.checked_add(1) {
                            leading_zero = temp
                        } else {
                            return self.make_error(format_args!("too many zeros"));
                        }
                    }
                    self.next_char();
                }
                '1'..='9' => {
                    let digit = c as u64 - '0' as u64;

                    // 如果 number 存在则 number=number*10+digit，否则 number=digit
                    if let Some(num) = number {
                        number = num.checked_mul(10);
                        if let Some(num) = number {
                            number = num.checked_add(digit);
                            if number.is_none() {
                                return self
                                    .make_error(format_args!("too large number {} * 10", num));
                            }
                        } else {
                            return self.make_error(format_args!("too large number {} * 10", num));
                        }
                    } else {
                        number = Some(digit);
                    }
                    self.next_char();
                }
                _ => break,
            }
        }

        let token: Token = Token::IntegerLiteral(leading_zero, number);
        Ok(Some(ParsedToken::new(token, start_location)))
    }

    /// Reads the next identifier or keyword token from the scanner
    fn next_id_kw(&mut self, first: char) -> Result<Option<ParsedToken>, TokenizeError> {
        let start_location: super::str_scanner::TokenLocation = self.location();

        let mut word: String = first.to_string();
        self.next_char(); // consume
        while let Some(c) = self.peek_char() {
            match c {
                'a'..='z' | 'A'..='Z' | '_' => {
                    word.push(c);
                    self.next_char();
                }
                _ => break,
            }
        }

        let token: Token = if word.len() > self.ley_word_max_length {
            Token::Identifier(word.as_str().into())
        } else {
            let upper: String = word.to_ascii_uppercase();
            if let Some(kw) = self.key_words.get(upper.as_str()) {
                Token::Keyword(*kw)
            } else {
                Token::Identifier(word.as_str().into())
            }
        };

        Ok(Some(ParsedToken::new(token, start_location)))
    }

    fn token_and_next(&mut self, token: Token) -> Result<Option<ParsedToken>, TokenizeError> {
        let start_location: super::str_scanner::TokenLocation = self.location();
        self.next_char();
        Ok(Some(ParsedToken::new(token, start_location)))
    }

    fn peek_char(&mut self) -> Option<char> {
        self.scanner.peek()
    }

    fn next_char(&mut self) {
        self.scanner.next()
    }

    fn location(&self) -> TokenLocation {
        self.scanner.location()
    }

    /// create tokenize-error
    fn make_error(&self, format_args: Arguments<'_>) -> Result<Option<ParsedToken>, TokenizeError> {
        Err(TokenizeError::new(
            format_args.to_string(),
            self.location(),
            self.sql,
        ))
    }
}

#[cfg(test)]
mod test {

    use std::fmt::Debug;

    use crate::sql::tokenizer::token::{Keyword, Token};

    use super::Tokenizer;

    #[test]
    fn select() {
        let tokens = Tokenizer::new("SELECT").tokenize().unwrap();
        let tokens: Vec<_> = tokens.tokens().iter().map(|t| t.token.clone()).collect();
        assert_eq(tokens, vec![Token::Keyword(Keyword::SELECT)]);
    }

    #[test]
    fn select_loc() {
        let tokens = Tokenizer::new("SELECT").tokenize().unwrap();
        assert_eq!(tokens.tokens()[0].location.column_number, 1);
    }

    #[test]
    fn select_1() {
        let tokens = Tokenizer::new("SELECT 1").tokenize().unwrap();
        let tokens: Vec<_> = tokens.tokens().iter().map(|t| t.token.clone()).collect();
        assert_eq(
            tokens,
            vec![
                Token::Keyword(Keyword::SELECT),
                Token::IntegerLiteral(0, Some(1)),
            ],
        );
    }

    #[test]
    fn select_1_loc() {
        let tokens = Tokenizer::new("SELECT 1").tokenize().unwrap();
        assert_eq!(tokens.tokens()[0].location.column_number, 1);
        assert_eq!(tokens.tokens()[1].location.column_number, 8);
    }

    #[test]
    fn select_123() {
        let tokens = Tokenizer::new("SELECT 123").tokenize().unwrap();
        let tokens: Vec<_> = tokens.tokens().iter().map(|t| t.token.clone()).collect();
        assert_eq(
            tokens,
            vec![
                Token::Keyword(Keyword::SELECT),
                Token::IntegerLiteral(0, Some(123)),
            ],
        );
    }

    #[test]
    fn select_0() {
        let tokens = Tokenizer::new("SELECT 0").tokenize().unwrap();
        let tokens: Vec<_> = tokens.tokens().iter().map(|t| t.token.clone()).collect();
        assert_eq(
            tokens,
            vec![
                Token::Keyword(Keyword::SELECT),
                Token::IntegerLiteral(1, None),
            ],
        );
    }

    #[test]
    fn select_00000() {
        let tokens = Tokenizer::new("SELECT 00000").tokenize().unwrap();
        let tokens: Vec<_> = tokens.tokens().iter().map(|t| t.token.clone()).collect();
        assert_eq(
            tokens,
            vec![
                Token::Keyword(Keyword::SELECT),
                Token::IntegerLiteral("00000".len() as u16, None),
            ],
        );
    }

    #[test]
    fn select_00123() {
        let tokens = Tokenizer::new("SELECT 00123").tokenize().unwrap();
        let tokens: Vec<_> = tokens.tokens().iter().map(|t| t.token.clone()).collect();
        assert_eq(
            tokens,
            vec![
                Token::Keyword(Keyword::SELECT),
                Token::IntegerLiteral(2, Some(123)),
            ],
        );
    }

    #[test]
    fn select_1_end() {
        let tokens = Tokenizer::new("SELECT 1;").tokenize().unwrap();
        let tokens: Vec<_> = tokens.tokens().iter().map(|t| t.token.clone()).collect();
        assert_eq(
            tokens,
            vec![
                Token::Keyword(Keyword::SELECT),
                Token::IntegerLiteral(0, Some(1)),
                Token::Semicolon,
            ],
        );
    }

    #[test]
    fn select_1_end_loc() {
        let tokens = Tokenizer::new("SELECT 1  ;").tokenize().unwrap();
        assert_eq!(tokens.tokens()[0].location.column_number, 1);
        assert_eq!(tokens.tokens()[1].location.column_number, 8);
        assert_eq!(tokens.tokens()[2].location.column_number, 11);
    }

    #[test]
    fn select_text() {
        let tokens = Tokenizer::new("SELECT 'hello';").tokenize().unwrap();
        assert_eq!(tokens.tokens()[0].location.column_number, 1);
        assert_eq!(tokens.tokens()[1].location.column_number, 8);
        assert_eq!(tokens.tokens()[2].location.column_number, 15);
        let tokens: Vec<_> = tokens.tokens().iter().map(|t| t.token.clone()).collect();
        assert_eq(
            tokens,
            vec![
                Token::Keyword(Keyword::SELECT),
                Token::StringLiteral("hello".into()),
                Token::Semicolon,
            ],
        );
    }

    #[test]
    fn select_text_escape() {
        let tokens = Tokenizer::new("SELECT '''he''llo''';").tokenize().unwrap();
        assert_eq!(tokens.tokens()[0].location.column_number, 1);
        assert_eq!(tokens.tokens()[1].location.column_number, 8);
        assert_eq!(tokens.tokens()[2].location.column_number, 21);
        let tokens: Vec<_> = tokens.tokens().iter().map(|t| t.token.clone()).collect();
        assert_eq(
            tokens,
            vec![
                Token::Keyword(Keyword::SELECT),
                Token::StringLiteral("'he'llo'".into()),
                Token::Semicolon,
            ],
        );
    }

    #[test]
    fn select_text_escape2() {
        assert!(Tokenizer::new("SELECT '''he\nllo''';").tokenize().is_err())
    }

    #[test]
    fn select_text_escape3() {
        let tokens = Tokenizer::new("SELECT '''he\\r\\nllo''';")
            .tokenize()
            .unwrap();
        assert_eq!(tokens.tokens()[0].location.column_number, 1);
        assert_eq!(tokens.tokens()[1].location.column_number, 8);
        assert_eq!(tokens.tokens()[2].location.column_number, 23);
        let tokens: Vec<_> = tokens.tokens().iter().map(|t| t.token.clone()).collect();
        assert_eq(
            tokens,
            vec![
                Token::Keyword(Keyword::SELECT),
                Token::StringLiteral("'he\r\nllo'".into()),
                Token::Semicolon,
            ],
        );
    }

    #[test]
    fn select_abc_from_def() {
        let tokens = Tokenizer::new("SELECT abc from dEf;").tokenize().unwrap();
        println!("[{}]", tokens);
        let tokens: Vec<_> = tokens.tokens().iter().map(|t| t.token.clone()).collect();
        assert_eq(
            tokens,
            vec![
                Token::Keyword(Keyword::SELECT),
                Token::Identifier("abc".into()),
                Token::Keyword(Keyword::FROM),
                Token::Identifier("dEf".into()),
                Token::Semicolon,
            ],
        );
    }

    #[test]
    fn select_abc_from_def_loc() {
        let tokens = Tokenizer::new("SELECT abc from dEf;").tokenize().unwrap();
        assert_eq!(tokens.tokens()[0].location.column_number, 1);
        assert_eq!(
            tokens.tokens()[1].location.column_number,
            1 + "SELECT ".len()
        );
        assert_eq!(
            tokens.tokens()[2].location.column_number,
            1 + "SELECT abc ".len()
        );
        assert_eq!(
            tokens.tokens()[3].location.column_number,
            1 + "SELECT abc from ".len()
        );
        assert_eq!(
            tokens.tokens()[4].location.column_number,
            1 + "SELECT abc from dEf".len()
        );
    }

    #[test]
    fn select_abc_from_def_where() {
        let tokens = Tokenizer::new("SELECT abc, kk, 1 from dEf where cc > 12;")
            .tokenize()
            .unwrap();
        println!("[{}]", tokens);
        let tokens: Vec<_> = tokens.tokens().iter().map(|t| t.token.clone()).collect();
        assert_eq(
            tokens,
            vec![
                Token::Keyword(Keyword::SELECT),
                Token::Identifier("abc".into()),
                Token::Comma,
                Token::Identifier("kk".into()),
                Token::Comma,
                Token::IntegerLiteral(0, Some(1)),
                Token::Keyword(Keyword::FROM),
                Token::Identifier("dEf".into()),
                Token::Keyword(Keyword::WHERE),
                Token::Identifier("cc".into()),
                Token::GreaterThan,
                Token::IntegerLiteral(0, Some(12)),
                Token::Semicolon,
            ],
        );
    }

    fn assert_eq<T: PartialEq + Debug>(value: T, expect: T) {
        println!("{:?}", value);
        assert_eq!(value, expect);
    }
}
