use std::{collections::HashMap, fmt::Arguments};

use strum::{EnumCount, IntoEnumIterator};

use super::{
    error::TokenizeError,
    token::{Keyword, ParsedToken, ParsedTokens, Token},
};

use super::str_scanner::Scanner;

pub struct Tokenizer {
    key_words: HashMap<String, Keyword>,
}

impl Tokenizer {
    pub fn new() -> Self {
        let mut key_words: HashMap<String, Keyword> = HashMap::with_capacity(Keyword::COUNT);
        Keyword::iter().for_each(|kw| {
            key_words.insert(kw.to_string(), kw);
        });
        Self { key_words }
    }

    /// tokenize the sql
    pub fn tokenize<'a>(&self, sql: &'a str) -> Result<ParsedTokens, TokenizeError> {
        let mut scanner: Scanner<'a> = Scanner::new(sql);
        let mut tokens: Vec<ParsedToken> = vec![];

        // 循环获取下一个 token
        loop {
            match self.next_token(&mut scanner) {
                Ok(token) => match token {
                    Some(t) => tokens.push(t),
                    None => break,
                },
                Err(e) => return Err(e),
            }
        }

        return Ok(ParsedTokens::new(tokens, sql.to_string()));
    }

    /// read next_token from scanner
    fn next_token(&self, scanner: &mut Scanner<'_>) -> Result<Option<ParsedToken>, TokenizeError> {
        loop {
            match scanner.peek() {
                Some(first) => match first {
                    'a'..='z' | 'A'..='Z' => return self.next_id_kw(scanner, first),
                    '0'..='9' => return self.next_number(scanner, first),
                    ' ' | '\r' | '\n' => scanner.next(),
                    ';' => return self.token_and_next(Token::Semicolon, scanner),
                    _ => return self.make_error(format_args!("unknown char {}", first), scanner),
                },
                None => return Ok(None),
            }
        }
    }

    /// Reads the next number
    fn next_number(
        &self,
        scanner: &mut Scanner<'_>,
        first: char,
    ) -> Result<Option<ParsedToken>, TokenizeError> {
        let start_location: super::str_scanner::TokenLocation = scanner.location();

        let mut leading_zero: u16 = 0;
        let mut number = None;

        let first: u64 = first as u64 - '0' as u64;
        if first == 0 {
            leading_zero += 1;
        } else {
            number = Some(first);
        }

        scanner.next(); // consume
        while let Some(c) = scanner.peek() {
            match c {
                '0' => {
                    // 如果 number 存在，则 number*=10，否则 leading_zero++
                    if let Some(num) = number {
                        number = num.checked_mul(10);
                        if number.is_none() {
                            return self.make_error(
                                format_args!("too large number {} * 10", num),
                                scanner,
                            );
                        }
                    } else {
                        if let Some(temp) = leading_zero.checked_add(1) {
                            leading_zero = temp
                        } else {
                            return self.make_error(format_args!("too many zeros"), scanner);
                        }
                    }
                    scanner.next();
                }
                '1'..='9' => {
                    let digit = c as u64 - '0' as u64;

                    // 如果 number 存在则 number=number*10+digit，否则 number=digit
                    if let Some(num) = number {
                        number = num.checked_mul(10);
                        if let Some(num) = number {
                            number = num.checked_add(digit);
                            if number.is_none() {
                                return self.make_error(
                                    format_args!("too large number {} * 10", num),
                                    scanner,
                                );
                            }
                        } else {
                            return self.make_error(
                                format_args!("too large number {} * 10", num),
                                scanner,
                            );
                        }
                    } else {
                        number = Some(digit);
                    }
                    scanner.next();
                }
                _ => break,
            }
        }

        let token: Token = Token::IntegerLiteral(leading_zero, number);
        Ok(Some(ParsedToken::new(token, start_location)))
    }

    /// Reads the next identifier or keyword token from the scanner
    fn next_id_kw(
        &self,
        scanner: &mut Scanner<'_>,
        first: char,
    ) -> Result<Option<ParsedToken>, TokenizeError> {
        let start_location: super::str_scanner::TokenLocation = scanner.location();

        let mut word: String = first.to_string();
        scanner.next(); // consume
        while let Some(c) = scanner.peek() {
            match c {
                'a'..='z' | 'A'..='Z' | '_' => {
                    word.push(c);
                    scanner.next();
                }
                _ => break,
            }
        }
        let token: Token = if let Some(kw) = self.key_words.get(&word.to_ascii_uppercase()) {
            Token::Keyword(*kw)
        } else {
            Token::Identifier(word)
        };

        Ok(Some(ParsedToken::new(token, start_location)))
    }

    fn token_and_next(
        &self,
        token: Token,
        scanner: &mut Scanner<'_>,
    ) -> Result<Option<ParsedToken>, TokenizeError> {
        let start_location: super::str_scanner::TokenLocation = scanner.location();
        scanner.next();
        Ok(Some(ParsedToken::new(token, start_location)))
    }

    /// create tokenize-error
    fn make_error(
        &self,
        format_args: Arguments<'_>,
        scanner: &Scanner<'_>,
    ) -> Result<Option<ParsedToken>, TokenizeError> {
        Err(TokenizeError::new(
            format_args.to_string(),
            scanner.location(),
            scanner.text().to_string(),
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
        let tokens = Tokenizer::new().tokenize("SELECT").unwrap();
        let tokens: Vec<_> = tokens.tokens().iter().map(|t| t.token.clone()).collect();
        assert_eq(tokens, vec![Token::Keyword(Keyword::SELECT)]);
    }

    #[test]
    fn select_loc() {
        let tokens = Tokenizer::new().tokenize("SELECT").unwrap();
        assert_eq!(tokens.tokens()[0].location.column_number, 1);
    }

    #[test]
    fn select_1() {
        let tokens = Tokenizer::new().tokenize("SELECT 1").unwrap();
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
        let tokens = Tokenizer::new().tokenize("SELECT 1").unwrap();
        assert_eq!(tokens.tokens()[0].location.column_number, 1);
        assert_eq!(tokens.tokens()[1].location.column_number, 8);
    }

    #[test]
    fn select_123() {
        let tokens = Tokenizer::new().tokenize("SELECT 123").unwrap();
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
        let tokens = Tokenizer::new().tokenize("SELECT 0").unwrap();
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
        let tokens = Tokenizer::new().tokenize("SELECT 00000").unwrap();
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
        let tokens = Tokenizer::new().tokenize("SELECT 00123").unwrap();
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
        let tokens = Tokenizer::new().tokenize("SELECT 1;").unwrap();
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
        let tokens = Tokenizer::new().tokenize("SELECT 1  ;").unwrap();
        assert_eq!(tokens.tokens()[0].location.column_number, 1);
        assert_eq!(tokens.tokens()[1].location.column_number, 8);
        assert_eq!(tokens.tokens()[2].location.column_number, 11);
    }

    #[test]
    fn select_abc_from_def() {
        let tokens = Tokenizer::new().tokenize("SELECT abc from dEf;").unwrap();
        println!("[{}]", tokens);
        let tokens: Vec<_> = tokens.tokens().iter().map(|t| t.token.clone()).collect();
        assert_eq(
            tokens,
            vec![
                Token::Keyword(Keyword::SELECT),
                Token::Identifier("abc".to_string()),
                Token::Keyword(Keyword::FROM),
                Token::Identifier("dEf".to_string()),
                Token::Semicolon,
            ],
        );
    }

    #[test]
    fn select_abc_from_def_loc() {
        let tokens = Tokenizer::new().tokenize("SELECT abc from dEf;").unwrap();
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

    fn assert_eq<T: PartialEq + Debug>(value: T, expect: T) {
        println!("{:?}", value);
        assert_eq!(value, expect);
    }
}
