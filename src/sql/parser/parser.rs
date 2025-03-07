use crate::sql::tokenizer::{
    str_scanner::TokenLocation,
    token::{Keyword, ParsedToken, ParsedTokens, Token},
};

use super::{
    ast::{leaf::Leaf, Statement, Statements},
    error::ParseError,
};

pub struct Parser {
    tokens: Box<[ParsedToken]>,
    index: usize,
    raw_sql: Box<str>,
}

impl Parser {
    pub fn new(tokens: ParsedTokens) -> Self {
        Self {
            tokens: tokens.tokens,
            raw_sql: tokens.raw_sql,
            index: 0,
        }
    }

    pub fn parse(mut self) -> Result<Statements, ParseError> {
        let mut statements: Vec<Statement> = Vec::new();
        while self.index < self.tokens.len() {
            statements.push(self.parse_statement()?);
        }
        Ok(Statements {
            statements: statements.into_boxed_slice(),
            raw_sql: self.raw_sql,
        })
    }

    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        match self.peek().unwrap().token {
            Token::Keyword(Keyword::SELECT) => self.parse_select(),
            Token::Semicolon => self.parse_empty_statement(),
            _ => Err(ParseError::new(
                "invalid statement",
                self.location().clone(),
                self.raw_sql.as_ref(),
            )),
        }
    }

    fn parse_select(&mut self) -> Result<Statement, ParseError> {
        assert_eq!(self.peek().unwrap().token, Token::Keyword(Keyword::SELECT));
        self.next(); // consume SELECT
        Err(ParseError::new(
            "invalid select statement",
            self.location().clone(),
            self.raw_sql.as_ref(),
        ))
    }

    fn parse_empty_statement(&mut self) -> Result<Statement, ParseError> {
        assert_eq!(self.peek().unwrap().token, Token::Semicolon);
        self.next(); // consume ;
        Ok(Statement::Empty(Leaf::new(&self.location())))
    }

    fn peek(&self) -> Option<&ParsedToken> {
        self.tokens.get(self.index)
    }

    fn next(&mut self) {
        if self.index < self.tokens.len() {
            self.index += 1;
        }
    }

    fn location(&self) -> &TokenLocation {
        let len: usize = self.tokens.len();
        if self.index < len {
            &self.tokens[self.index].location
        } else {
            &self.tokens[len - 1].location
        }
    }
}

#[cfg(test)]
mod test {
    use crate::sql::tokenizer::tokenizer::Tokenizer;

    use super::*;

    /// test empty statement
    #[test]
    fn test_empty_statement() {
        let tokens: ParsedTokens = Tokenizer::new().tokenize(";").unwrap();
        let parser: Parser = Parser::new(tokens.clone());
        let statements: Statements = parser.parse().unwrap();
        assert_eq!(statements.statements.len(), 1);
        assert_eq!(
            statements.statements[0],
            Statement::Empty(Leaf::new(&tokens.tokens[0].location))
        );
    }
}
