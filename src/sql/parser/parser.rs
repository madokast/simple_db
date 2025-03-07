use crate::sql::tokenizer::{str_scanner::TokenLocation, token::{ParsedToken, ParsedTokens}};

use super::{ast::{Statement, Statements}, error::ParseError};

pub struct Parser {
    tokens: Vec<ParsedToken>,
    index: usize,
    raw_sql: String,
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
            let statement: Statement = self.parse_statement()?;
            statements.push(statement);
        }
        Ok(Statements {
            statements,
            raw_sql: self.raw_sql,
        })
    }

    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        match self.peek() {
            _ => {
                self.next();
            }
        }
        Err(ParseError::new(
            "Not implemented".to_string(),
            *self.location(),
            self.raw_sql.clone(),
        ))
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
            &self.tokens[len-1].location
        }
    }

}
