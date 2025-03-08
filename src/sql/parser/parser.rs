use std::{fmt::Arguments, rc::Rc};

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
        literal::{Literal, Value},
        select::{Expression, SelectItem},
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
                        _ => {
                            return self
                                .make_error(format_args!("invalid token {token}, expect keyword"))
                        }
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
                Token::Multiply => {
                    let w: SelectItem = SelectItem::Wildcard(Leaf::new(&self.location()));
                    self.next(); // consume *
                    Ok(w)
                }
                _ => Ok(SelectItem::Expression(self.parse_expression()?)),
            },
            None => self.make_error(format_args!("unexpected end of input, expect select-item")),
        }
    }

    fn parse_expression(&mut self) -> Result<Expression, ParseError> {
        match self.peek() {
            Some(token) => match &token.token {
                Token::Identifier(_) => Ok(Expression::Identifier(self.parse_identifier()?)),
                Token::StringLiteral(s) => {
                    let expr: Expression = Expression::Literal(Literal {
                        value: Value::String(Rc::clone(&s)),
                        leaf: Leaf::new(&token.location),
                    });
                    self.next(); // consume string literal
                    Ok(expr)
                }
                Token::IntegerLiteral(zeros, number) => {
                    Ok(Expression::Literal(self.parse_number(*zeros, *number)?))
                }
                _ => self.make_error(format_args!("invalid token {token}, expect expression")),
            },
            None => self.make_error(format_args!("unexpected end of input, expect select-item")),
        }
    }

    fn parse_number(&mut self, _zeros: u16, number: Option<u64>) -> Result<Literal, ParseError> {
        assert_eq!(
            self.peek().unwrap().token,
            Token::IntegerLiteral(_zeros, number)
        );
        let leaf: Leaf = Leaf::new(self.location());
        self.next();

        let integer: u64 = number.unwrap_or(0);
        if self.next_if(|t| *t == Token::Period) {
            match self.peek() {
                Some(token) => match &token.token {
                    Token::IntegerLiteral(zeros, number) => {
                        match number {
                            Some(fraction) => {
                                // SELECT 1.1
                                let number: f64 = self.parse_float(integer, *zeros, *fraction)?;
                                self.next(); // consume fraction
                                Ok(Literal {
                                    value: Value::Float(number),
                                    leaf,
                                })
                            }
                            None => {
                                // SELECT 1.000
                                self.next(); // consume 0 fraction
                                Ok(Literal {
                                    value: Value::Float(integer as f64),
                                    leaf,
                                })
                            }
                        }
                    }
                    _ => Ok(Literal {
                        // SELECT 1. [FROM
                        value: Value::Integer(integer),
                        leaf,
                    }),
                },
                None => Ok(Literal {
                    // SELECT 1. [EOF
                    value: Value::Integer(integer),
                    leaf,
                }),
            }
        } else {
            // SELECT 1
            Ok(Literal {
                value: Value::Integer(integer),
                leaf,
            })
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
                _ => self.make_error(format_args!("invalid token {token}, expect identifier")),
            },
            None => self.make_error(format_args!("unexpected end of input, expect identifier")),
        }
    }

    fn parse_float(&self, integer: u64, zeros: u16, fraction: u64) -> Result<f64, ParseError> {
        let s: String = format!(
            "{integer}.{:0>width$}{fraction}",
            "",
            width = zeros as usize
        );
        s.parse().map_err(|e: std::num::ParseFloatError| {
            ParseError::new(
                e.to_string(),
                self.location().clone(),
                self.raw_sql.as_ref(),
            )
        })
    }

    fn parse_empty_statement(&mut self) -> Result<Statement, ParseError> {
        assert_eq!(self.peek().unwrap().token, Token::Semicolon);
        self.next(); // consume ;
        Ok(Statement::Empty(Leaf::new(&self.location())))
    }

    fn peek(&self) -> Option<&ParsedToken> {
        self.tokens.get(self.index)
    }

    fn next_if(&mut self, predicate: impl FnOnce(&Token) -> bool) -> bool {
        if let Some(token) = self.peek() {
            if predicate(&token.token) {
                self.next(); // consume
                return true;
            }
        }
        false
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
            format_args.to_string(),
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
                items: vec![SelectItem::Expression(Expression::Identifier(
                    Identifier::SingleIdentifier(SingleIdentifier {
                        value: "a".into(),
                        leaf: Leaf::new(&tokens.tokens[1].location),
                    })
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
                items: vec![SelectItem::Expression(Expression::Identifier(
                    Identifier::SingleIdentifier(SingleIdentifier {
                        value: "abcABCdef".into(),
                        leaf: Leaf::new(&tokens.tokens[1].location),
                    })
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
                items: vec![SelectItem::Expression(Expression::Identifier(
                    Identifier::CombinedIdentifier(
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
                    )
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
        let tokens: ParsedTokens = Tokenizer::new()
            .tokenize(" SELECT    a, * ,  b.c ;  ")
            .unwrap();
        println!("{:#?}", tokens);
        let mut parser: Parser<'_> = Parser::new(&tokens);
        let statements: Statements = parser.parse().unwrap();
        // println!("{:?}", statements);
        assert_eq!(statements.statements.len(), 1);
        assert_eq!(
            statements.statements[0],
            Statement::Select(Box::new(Select {
                items: vec![
                    SelectItem::Expression(Expression::Identifier(Identifier::SingleIdentifier(
                        SingleIdentifier {
                            value: "a".into(),
                            leaf: Leaf::new(&tokens.tokens[1].location),
                        }
                    ))),
                    SelectItem::Wildcard(Leaf::new(&tokens.tokens[3].location)),
                    SelectItem::Expression(Expression::Identifier(Identifier::CombinedIdentifier(
                        vec![
                            SingleIdentifier {
                                value: "b".into(),
                                leaf: Leaf::new(&tokens.tokens[5].location),
                            },
                            SingleIdentifier {
                                value: "c".into(),
                                leaf: Leaf::new(&tokens.tokens[7].location),
                            },
                        ]
                        .into_boxed_slice()
                    )))
                ]
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
    fn select_str() {
        let tokens: ParsedTokens = Tokenizer::new().tokenize("SELECT 'hello'").unwrap();
        let mut parser: Parser<'_> = Parser::new(&tokens);
        let statements: Statements = parser.parse().unwrap();
        // println!("{:?}", statements);
        assert_eq!(statements.statements.len(), 1);
        assert_eq!(
            statements.statements[0],
            Statement::Select(Box::new(Select {
                items: vec![SelectItem::Expression(Expression::Literal(Literal {
                    value: Value::String("hello".into()),
                    leaf: Leaf::new(&tokens.tokens[1].location)
                }))]
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
    fn select_str2() {
        let tokens: ParsedTokens = Tokenizer::new()
            .tokenize("SELECT 'hello', 'world!\\n';")
            .unwrap();
        let mut parser: Parser<'_> = Parser::new(&tokens);
        let statements: Statements = parser.parse().unwrap();
        // println!("{:?}", statements);
        assert_eq!(statements.statements.len(), 1);
        assert_eq!(
            statements.statements[0],
            Statement::Select(Box::new(Select {
                items: vec![
                    SelectItem::Expression(Expression::Literal(Literal {
                        value: Value::String("hello".into()),
                        leaf: Leaf::new(&tokens.tokens[1].location)
                    })),
                    SelectItem::Expression(Expression::Literal(Literal {
                        value: Value::String("world!\n".into()),
                        leaf: Leaf::new(&tokens.tokens[3].location)
                    })),
                ]
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
    fn select_int() {
        let tokens: ParsedTokens = Tokenizer::new()
            .tokenize("SELECT 0, 00, 123, 001100;")
            .unwrap();
        let mut parser: Parser<'_> = Parser::new(&tokens);
        let statements: Statements = parser.parse().unwrap();
        // println!("{:?}", statements);
        assert_eq!(statements.statements.len(), 1);
        assert_eq!(
            statements.statements[0],
            Statement::Select(Box::new(Select {
                items: vec![
                    SelectItem::Expression(Expression::Literal(Literal {
                        value: Value::Integer(0),
                        leaf: Leaf::new(&tokens.tokens[1].location)
                    })),
                    SelectItem::Expression(Expression::Literal(Literal {
                        value: Value::Integer(0),
                        leaf: Leaf::new(&tokens.tokens[3].location)
                    })),
                    SelectItem::Expression(Expression::Literal(Literal {
                        value: Value::Integer(123),
                        leaf: Leaf::new(&tokens.tokens[5].location)
                    })),
                    SelectItem::Expression(Expression::Literal(Literal {
                        value: Value::Integer(1100),
                        leaf: Leaf::new(&tokens.tokens[7].location)
                    })),
                ]
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
    fn select_float() {
        let tokens: ParsedTokens = Tokenizer::new()
            .tokenize("SELECT 1.0, 1.25, 0.625, 3.0625")
            .unwrap();
        let mut parser: Parser<'_> = Parser::new(&tokens);
        let statements: Statements = parser.parse().unwrap();
        // println!("{:?}", statements);
        assert_eq!(statements.statements.len(), 1);
        assert_eq!(
            statements.statements[0],
            Statement::Select(Box::new(Select {
                items: vec![
                    SelectItem::Expression(Expression::Literal(Literal {
                        value: Value::Float(1.0),
                        leaf: Leaf::new(&tokens.tokens[1].location)
                    })),
                    SelectItem::Expression(Expression::Literal(Literal {
                        value: Value::Float(1.25),
                        leaf: Leaf::new(&tokens.tokens[5].location)
                    })),
                    SelectItem::Expression(Expression::Literal(Literal {
                        value: Value::Float(0.625),
                        leaf: Leaf::new(&tokens.tokens[9].location)
                    })),
                    SelectItem::Expression(Expression::Literal(Literal {
                        value: Value::Float(3.0625),
                        leaf: Leaf::new(&tokens.tokens[13].location)
                    })),
                ]
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
    fn select_int_int_float() {
        let tokens: ParsedTokens = Tokenizer::new()
            .tokenize("SELECT 1.0, 1.25, 0.625, 3.0625, 123")
            .unwrap();
        let mut parser: Parser<'_> = Parser::new(&tokens);
        let statements: Statements = parser.parse().unwrap();
        // println!("{:?}", statements);
        assert_eq!(statements.statements.len(), 1);
        assert_eq!(
            statements.statements[0],
            Statement::Select(Box::new(Select {
                items: vec![
                    SelectItem::Expression(Expression::Literal(Literal {
                        value: Value::Float(1.0),
                        leaf: Leaf::new(&tokens.tokens[1].location)
                    })),
                    SelectItem::Expression(Expression::Literal(Literal {
                        value: Value::Float(1.25),
                        leaf: Leaf::new(&tokens.tokens[5].location)
                    })),
                    SelectItem::Expression(Expression::Literal(Literal {
                        value: Value::Float(0.625),
                        leaf: Leaf::new(&tokens.tokens[9].location)
                    })),
                    SelectItem::Expression(Expression::Literal(Literal {
                        value: Value::Float(3.0625),
                        leaf: Leaf::new(&tokens.tokens[13].location)
                    })),
                    SelectItem::Expression(Expression::Literal(Literal {
                        value: Value::Integer(123),
                        leaf: Leaf::new(&tokens.tokens[17].location)
                    })),
                ]
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
}
