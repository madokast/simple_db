use std::fmt::Arguments;

use crate::sql::{
    parser::ast::Select,
    tokenizer::{
        str_scanner::TokenLocation,
        token::{Keyword, ParsedToken, ParsedTokens, Token},
    },
};

use super::{
    ast::{
        identifier::{Identifier, SingleIdentifier},
        leaf::Leaf,
        select::SelectItem,
        Statement, Statements,
    },
    error::ParseError,
};

pub struct Parser<'a> {
    tokens: &'a [ParsedToken],
    index: usize,
    raw_sql: &'a str,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a ParsedTokens) -> Self {
        Self {
            tokens: tokens.tokens.as_ref(),
            raw_sql: tokens.raw_sql.as_ref(),
            index: 0,
        }
    }

    pub fn parse(&mut self) -> Result<Statements, ParseError> {
        let mut statements: Vec<Statement> = Vec::new();
        while self.index < self.tokens.len() {
            statements.push(self.parse_statement()?);
        }
        Ok(Statements {
            statements: statements.into_boxed_slice(),
            raw_sql: self.raw_sql.into(),
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

        let select_items: Box<[SelectItem]> = self.parse_select_items()?;

        // consume Semicolon ; if exists
        self.next_if(|t| *t == Token::Semicolon);

        Ok(Statement::Select(Box::new(Select {
            items: select_items,
            from: vec![].into_boxed_slice(),
            wheres: None,
            order_by: vec![].into_boxed_slice(),
            group_by: vec![].into_boxed_slice(),
            limit: None,
            offset: None,
        })))
    }

    fn parse_select_items(&mut self) -> Result<Box<[SelectItem]>, ParseError> {
        let mut items: Vec<SelectItem> = Vec::new();
        loop {
            items.push(self.parse_select_item()?);
            match self.peek() {
                Some(token) => {
                    match token.token {
                        Token::Comma => self.next(), // consume
                        Token::Semicolon => break,
                        Token::Keyword(kw) => match kw {
                            Keyword::FROM | Keyword::WHERE => break,
                            _ => {
                                return self
                                    .make_error(format_args!("invalid keyword {kw}, expect FROM"))
                            }
                        },
                        _ => return self.make_error(format_args!("invalid token {token}")),
                    }
                }
                None => break,
            }
        }
        Ok(items.into_boxed_slice())
    }

    fn parse_select_item(&mut self) -> Result<SelectItem, ParseError> {
        match self.peek() {
            Some(token) => match &token.token {
                Token::Identifier(_) => Ok(SelectItem::Identifier(self.parse_identifier()?)),
                Token::Multiply => {
                    let w: SelectItem = SelectItem::Wildcard(Leaf::new(&self.location()));
                    self.next(); // consume *
                    Ok(w)
                }
                _ => self.make_error(format_args!("invalid token {token}")),
            },
            None => self.make_error(format_args!("unexpected end of input, expect select-item")),
        }
    }

    /// parse identifier like single col_name or combined tab_name.col_name
    /// TODO handle identifier with wildcard schema.table.*
    fn parse_identifier(&mut self) -> Result<Identifier, ParseError> {
        match self.peek() {
            Some(token) => match &token.token {
                Token::Identifier(ident) => {
                    let mut identifiers: Vec<SingleIdentifier> = vec![SingleIdentifier {
                        value: ident.clone(),
                        leaf: Leaf::new(&token.location),
                    }];
                    self.next(); // consume identifier
                    while let Some(token) = self.peek() {
                        if token.token == Token::Period {
                            self.next(); // consume.
                            match self.peek() {
                                Some(token) => match &token.token {
                                    Token::Identifier(ident) => {
                                        identifiers.push(SingleIdentifier {
                                            value: ident.clone(),
                                            leaf: Leaf::new(&token.location),
                                        });
                                        self.next(); // consume identifier
                                    }
                                    _ => {
                                        return self.make_error(format_args!(
                                            "invalid token {token}, expect identifier"
                                        ))
                                    }
                                },
                                None => {
                                    return self.make_error(format_args!(
                                        "unexpected end of input, expect identifier"
                                    ))
                                }
                            }
                        } else {
                            break;
                        }
                    }
                    if identifiers.len() == 1 {
                        Ok(Identifier::SingleIdentifier(identifiers.pop().unwrap()))
                    } else {
                        Ok(Identifier::CombinedIdentifier(
                            identifiers.into_boxed_slice(),
                        ))
                    }
                }
                _ => self.make_error(format_args!("invalid token {token}")),
            },
            None => self.make_error(format_args!("unexpected end of input, expect identifier")),
        }
    }

    fn parse_empty_statement(&mut self) -> Result<Statement, ParseError> {
        assert_eq!(self.peek().unwrap().token, Token::Semicolon);
        self.next(); // consume ;
        Ok(Statement::Empty(Leaf::new(&self.location())))
    }

    fn peek(&self) -> Option<&ParsedToken> {
        self.tokens.get(self.index)
    }

    fn next_if(&mut self, predicate: impl FnOnce(&Token) -> bool) {
        if let Some(token) = self.peek() {
            if predicate(&token.token) {
                self.next(); // consume
            }
        }
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

    fn make_error<T>(&self, format_args: Arguments) -> Result<T, ParseError> {
        Err(ParseError::new(
            &format_args.to_string(),
            self.location().clone(),
            self.raw_sql.as_ref(),
        ))
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
        let mut parser: Parser<'_> = Parser::new(&tokens);
        let statements: Statements = parser.parse().unwrap();
        assert_eq!(statements.statements.len(), 1);
        assert_eq!(
            statements.statements[0],
            Statement::Empty(Leaf::new(&tokens.tokens[0].location))
        );
    }

    #[test]
    fn select_a() {
        let tokens: ParsedTokens = Tokenizer::new().tokenize("SELECT a;").unwrap();
        let mut parser: Parser<'_> = Parser::new(&tokens);
        let statements: Statements = parser.parse().unwrap();
        // println!("{:?}", statements);
        assert_eq!(statements.statements.len(), 1);
        assert_eq!(
            statements.statements[0],
            Statement::Select(Box::new(Select {
                items: vec![SelectItem::Identifier(Identifier::SingleIdentifier(
                    SingleIdentifier {
                        value: "a".into(),
                        leaf: Leaf::new(&tokens.tokens[1].location),
                    }
                ))]
                .into_boxed_slice(),
                from: vec![].into_boxed_slice(),
                wheres: None,
                order_by: vec![].into_boxed_slice(),
                group_by: vec![].into_boxed_slice(),
                limit: None,
                offset: None,
            }))
        )
    }

    #[test]
    fn select_abc_abc_def() {
        let tokens: ParsedTokens = Tokenizer::new().tokenize("SELECT abcABCdef;").unwrap();
        let mut parser: Parser<'_> = Parser::new(&tokens);
        let statements: Statements = parser.parse().unwrap();
        // println!("{:?}", statements);
        assert_eq!(statements.statements.len(), 1);
        assert_eq!(
            statements.statements[0],
            Statement::Select(Box::new(Select {
                items: vec![SelectItem::Identifier(Identifier::SingleIdentifier(
                    SingleIdentifier {
                        value: "abcABCdef".into(),
                        leaf: Leaf::new(&tokens.tokens[1].location),
                    }
                ))]
                .into_boxed_slice(),
                from: vec![].into_boxed_slice(),
                wheres: None,
                order_by: vec![].into_boxed_slice(),
                group_by: vec![].into_boxed_slice(),
                limit: None,
                offset: None,
            }))
        )
    }

    #[test]
    fn select_wildcard() {
        let tokens: ParsedTokens = Tokenizer::new().tokenize("SELECT *").unwrap();
        let mut parser: Parser<'_> = Parser::new(&tokens);
        let statements: Statements = parser.parse().unwrap();
        // println!("{:?}", statements);
        assert_eq!(statements.statements.len(), 1);
        assert_eq!(
            statements.statements[0],
            Statement::Select(Box::new(Select {
                items: vec![SelectItem::Wildcard(Leaf::new(&tokens.tokens[1].location))]
                    .into_boxed_slice(),
                from: vec![].into_boxed_slice(),
                wheres: None,
                order_by: vec![].into_boxed_slice(),
                group_by: vec![].into_boxed_slice(),
                limit: None,
                offset: None,
            }))
        )
    }

    #[test]
    fn select_abc_abc_def_dot() {
        let tokens: ParsedTokens = Tokenizer::new().tokenize("SELECT abc.ABC.def;").unwrap();
        let mut parser: Parser<'_> = Parser::new(&tokens);
        let statements: Statements = parser.parse().unwrap();
        // println!("{:?}", statements);
        assert_eq!(statements.statements.len(), 1);
        assert_eq!(
            statements.statements[0],
            Statement::Select(Box::new(Select {
                items: vec![SelectItem::Identifier(Identifier::CombinedIdentifier(
                    vec![
                        SingleIdentifier {
                            value: "abc".into(),
                            leaf: Leaf::new(&tokens.tokens[1].location),
                        },
                        SingleIdentifier {
                            value: "ABC".into(),
                            leaf: Leaf::new(&tokens.tokens[3].location),
                        },
                        SingleIdentifier {
                            value: "def".into(),
                            leaf: Leaf::new(&tokens.tokens[5].location),
                        },
                    ]
                    .into_boxed_slice()
                ))]
                .into_boxed_slice(),
                from: vec![].into_boxed_slice(),
                wheres: None,
                order_by: vec![].into_boxed_slice(),
                group_by: vec![].into_boxed_slice(),
                limit: None,
                offset: None,
            }))
        )
    }

    #[test]
    fn select_many() {
        let tokens: ParsedTokens = Tokenizer::new().tokenize(" SELECT    a, * ,  b.c ;  ").unwrap();
        println!("{:#?}", tokens);
        let mut parser: Parser<'_> = Parser::new(&tokens);
        let statements: Statements = parser.parse().unwrap();
        // println!("{:?}", statements);
        assert_eq!(statements.statements.len(), 1);
        assert_eq!(
            statements.statements[0],
            Statement::Select(Box::new(Select {
                items: vec![
                        SelectItem::Identifier(Identifier::SingleIdentifier(
                            SingleIdentifier {
                                value: "a".into(),
                                leaf: Leaf::new(&tokens.tokens[1].location),
                            }
                        )),
                        SelectItem::Wildcard(Leaf::new(&tokens.tokens[3].location)),
                        SelectItem::Identifier(Identifier::CombinedIdentifier(
                            vec![
                                SingleIdentifier {
                                    value: "b".into(),
                                    leaf: Leaf::new(&tokens.tokens[5].location),
                                },
                                SingleIdentifier {
                                    value: "c".into(),
                                    leaf: Leaf::new(&tokens.tokens[7].location),
                                },
                            ].into_boxed_slice()
                        ))
                    ].into_boxed_slice(),
                from: vec![].into_boxed_slice(),
                wheres: None,
                order_by: vec![].into_boxed_slice(),
                group_by: vec![].into_boxed_slice(),
                limit: None,
                offset: None,
            }))
        )
    }

}
