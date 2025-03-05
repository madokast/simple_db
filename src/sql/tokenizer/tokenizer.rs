use std::collections::HashMap;

use strum::{EnumCount, IntoEnumIterator};

use super::{
    error::TokenizeError,
    token::{Keyword, ParsedToken, ParsedTokens},
};

use super::str_scanner::Scanner;

pub struct Tokenizer {
    key_words: HashMap<String, Keyword>,
}

impl Tokenizer {
    pub fn new() -> Self {
        let mut key_words = HashMap::with_capacity(Keyword::COUNT);
        Keyword::iter().for_each(|kw| {
            key_words.insert(kw.to_string(), kw);
        });
        Self { key_words }
    }

    pub fn tokenize(&self, sql: &str) -> Result<ParsedTokens, TokenizeError> {
        let mut scanner: Scanner<'_> = Scanner::new(sql);
        let mut tokens: Vec<ParsedToken> = vec![];

        loop {
            match self.next_token(&mut scanner) {
                Ok(token) => match token {
                    Some(t) => {
                        tokens.push(t);
                    }
                    None => break,
                },
                Err(e) => {
                    return Err(e);
                }
            }
        }

        return Ok(ParsedTokens::new(vec![], sql.to_string()));
    }

    fn next_token(&self, scanner: &mut Scanner<'_>) -> Result<Option<ParsedToken>, TokenizeError> {
        _ = self.key_words[&"SELECT".to_string()];
        _ = scanner.peek();
        _ = scanner.next();
        return Err(TokenizeError::new(
            "message".to_string(),
            scanner.location(),
            scanner.text().to_string(),
        ));
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
