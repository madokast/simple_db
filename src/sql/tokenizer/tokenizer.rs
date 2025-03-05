use std::collections::HashMap;

use strum::{EnumCount, IntoEnumIterator};

use super::{error::Error, token::{Keyword, ParsedToken, ParsedTokens}};

use super::str_scanner::Scanner;

pub struct Tokenizer {
    key_words:HashMap<String, Keyword>
}

impl Tokenizer {
    pub fn new() -> Self {
        let mut key_words = HashMap::with_capacity(Keyword::COUNT);
        Keyword::iter().for_each(|kw| {key_words.insert(kw.to_string(), kw);});
        Self { key_words }
    }

    pub fn tokenize(&self, sql: &str) -> Result<ParsedTokens, Error> {
        let mut scanner: Scanner<'_> = Scanner::new(sql);
        let mut tokens: Vec<ParsedToken> = vec![];

        loop {
            match self.next_token(&mut scanner) {
                Ok(token) => {
                    match token {
                        Some(t) => {
                            tokens.push(t);
                        }, None => break
                    }
                }, Err(e) => {
                    return Err(e);
                }
            }
        }

        return Ok(ParsedTokens::new(vec![], sql.to_string()));
    }

    fn next_token(&self, scanner: &mut Scanner<'_>) -> Result<Option<ParsedToken>, Error> {
        _ = self.key_words[&"SELECT".to_string()];
        _ = scanner.peek();
        _ = scanner.next();
        return Err(Error::new("message".to_string(), scanner.location()));
    }
}

#[cfg(test)]
mod test {
    use super::Tokenizer;

    #[test]
    fn select_1() {
        let _ = Tokenizer::new().tokenize("SELECT 1");
    }
}