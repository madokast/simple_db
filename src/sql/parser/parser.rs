use std::{fmt::Arguments, rc::Rc};

use crate::sql::{
    parser::ast::{select::FromItem, Select},
    tokenizer::{
        str_scanner::TokenLocation,
        token::{Keyword, ParsedToken, ParsedTokens, Token},
    },
};

use super::{
    ast::{
        expression::{
            Alias, BinaryExpression, BinaryOperator, Expression, Function, UnaryExpression,
            UnaryOperator,
        },
        identifier::{Identifier, SingleIdentifier},
        leaf::Leaf,
        literal::{Literal, Value},
        select::{OrderBy, SelectItem},
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
            self.next_if(|t| *t == Token::Semicolon);
        }
        Ok(Statements {
            statements: statements.into_boxed_slice(),
            raw_sql: self.raw_sql.into(),
        })
    }

    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        match self.peek().unwrap().token {
            Token::Keyword(Keyword::SELECT) => Ok(Statement::Select(self.parse_select()?)),
            Token::Semicolon => self.parse_empty_statement(),
            _ => Err(ParseError::new(
                "invalid statement",
                self.location().clone(),
                self.raw_sql.as_ref(),
            )),
        }
    }

    fn parse_select(&mut self) -> Result<Select, ParseError> {
        debug_assert_eq!(self.peek().unwrap().token, Token::Keyword(Keyword::SELECT));
        self.next(); // consume SELECT

        let select_items: Box<[SelectItem]> = self.parse_select_items()?;

        let from: Box<[FromItem]> = self.parse_from()?;

        let wheres: Option<Expression> = self.parse_where()?;

        let group_by: Box<[Identifier]> = self.parse_group_by()?;

        let having: Option<Expression> = self.parse_having()?;

        let order_by: Box<[OrderBy]> = self.parse_order_by()?;

        Ok(Select {
            items: select_items,
            from: from,
            wheres: wheres,
            group_by: group_by,
            having: having,
            order_by: order_by,
            limit: None,
            offset: None,
        })
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
                            Keyword::FROM
                            | Keyword::WHERE
                            | Keyword::HAVING
                            | Keyword::GROUP
                            | Keyword::ORDER => break,
                            _ => {
                                return self
                                    .make_error(format_args!("invalid keyword {kw}, expect FROM"))
                            }
                        },
                        _ => {
                            return self.make_error(format_args!(
                                "invalid token {token}, expect keyword FROM"
                            ))
                        }
                    }
                }
                None => break,
            }
        }
        Ok(items.into_boxed_slice())
    }

    fn parse_from(&mut self) -> Result<Box<[FromItem]>, ParseError> {
        if self.next_if(|t| *t == Token::Keyword(Keyword::FROM)) {
            let mut from: Vec<FromItem> = vec![];
            loop {
                let expression: Expression = self.parse_expression(0)?;
                let alias: Option<Identifier> =
                    if self.next_if(|t| *t == Token::Keyword(Keyword::AS)) {
                        Some(self.parse_identifier()?)
                    } else {
                        None
                    };
                from.push(FromItem {
                    expression: expression,
                    alias: alias,
                });
                if !self.next_if(|t| *t == Token::Comma) {
                    break;
                }
            }
            Ok(from.into_boxed_slice())
        } else {
            Ok(vec![].into_boxed_slice())
        }
    }

    fn parse_where(&mut self) -> Result<Option<Expression>, ParseError> {
        if self.next_if(|t| *t == Token::Keyword(Keyword::WHERE)) {
            Ok(Some(self.parse_expression(0)?))
        } else {
            Ok(None)
        }
    }

    fn parse_having(&mut self) -> Result<Option<Expression>, ParseError> {
        if self.next_if(|t| *t == Token::Keyword(Keyword::HAVING)) {
            Ok(Some(self.parse_expression(0)?))
        } else {
            Ok(None)
        }
    }

    fn parse_group_by(&mut self) -> Result<Box<[Identifier]>, ParseError> {
        let mut group_by: Vec<Identifier> = vec![];
        if self.next_if(|t| *t == Token::Keyword(Keyword::GROUP)) {
            if self.next_if(|t| *t == Token::Keyword(Keyword::BY)) {
                loop {
                    group_by.push(self.parse_identifier()?);
                    if !self.next_if(|t| *t == Token::Comma) {
                        break;
                    }
                }
            } else {
                return self.make_error(format_args!("expect keyword BY of GROUP BY"));
            }
        }
        Ok(group_by.into_boxed_slice())
    }

    fn parse_order_by(&mut self) -> Result<Box<[OrderBy]>, ParseError> {
        let mut order_by: Vec<OrderBy> = vec![];

        if self.next_if(|t| *t == Token::Keyword(Keyword::ORDER)) {
            if self.next_if(|t| *t == Token::Keyword(Keyword::BY)) {
                loop {
                    let identifier: Identifier = self.parse_identifier()?;
                    let asc: bool = if self.next_if(|t| *t == Token::Keyword(Keyword::ASC)) {
                        true
                    } else if self.next_if(|t| *t == Token::Keyword(Keyword::DESC)) {
                        false
                    } else {
                        true
                    };

                    order_by.push(OrderBy {
                        identifier: identifier,
                        asc: asc,
                    });
                }
            } else {
                return self.make_error(format_args!("expect keyword BY of ORDER BY"));
            }
        }
        Ok(order_by.into_boxed_slice())
    }

    fn parse_select_item(&mut self) -> Result<SelectItem, ParseError> {
        let expr: Expression = self.parse_expression(0)?;

        match self.peek() {
            Some(token) => {
                match &token.token {
                    Token::Keyword(Keyword::AS) => {
                        self.next(); // consume AS
                        let alias: Identifier = self.parse_identifier()?;
                        Ok(SelectItem::Alias(Alias {
                            expression: expr,
                            alias,
                        }))
                    }
                    Token::Identifier(ident) => {
                        let alias: Identifier = Identifier::Single(SingleIdentifier {
                            value: ident.clone(),
                            leaf: Leaf::new(&token.location),
                        });
                        self.next();
                        Ok(SelectItem::Alias(Alias {
                            expression: expr,
                            alias,
                        }))
                    }
                    _ => Ok(SelectItem::Expression(expr)),
                }
            }
            None => Ok(SelectItem::Expression(expr)),
        }
    }

    fn parse_expression(&mut self, priority: usize) -> Result<Expression, ParseError> {
        // 前缀表达式
        let prefix: Option<UnaryOperator> = self.parse_prefix_operator()?;

        // 表达式左侧
        let mut left: Expression = self.parse_expression_operand()?;

        // 补上前缀
        if let Some(op) = prefix {
            left = Expression::UnaryExpression(UnaryExpression {
                operator: op,
                expression: Box::new(left),
            })
        }

        loop {
            let operator: Option<BinaryOperator> = self.peek_binary_operator()?;
            match operator {
                Some(op) => {
                    if priority < op.priority() {
                        self.next(); // consume operator
                        let right: Expression = self.parse_expression(op.priority())?;
                        left = Expression::BinaryExpression(BinaryExpression {
                            left: Box::new(left),
                            operator: op,
                            right: Box::new(right),
                        })
                    } else {
                        return Ok(left);
                    }
                }
                None => {
                    return Ok(left);
                }
            }
        }
    }

    /// parse prefix operator + - NOT
    fn parse_prefix_operator(&mut self) -> Result<Option<UnaryOperator>, ParseError> {
        match self.peek() {
            Some(token) => match token.token {
                Token::Plus => Ok(Some(UnaryOperator::Plus(Leaf::new(
                    &self.location_and_next(),
                )))),
                Token::Minus => Ok(Some(UnaryOperator::Minus(Leaf::new(
                    &self.location_and_next(),
                )))),
                Token::Keyword(Keyword::NOT) => Ok(Some(UnaryOperator::NOT(Leaf::new(
                    &self.location_and_next(),
                )))),
                _ => Ok(None),
            },
            None => Ok(None),
        }
    }

    fn parse_expression_operand(&mut self) -> Result<Expression, ParseError> {
        match self.peek() {
            Some(token) => match &token.token {
                Token::Keyword(kw) => match kw {
                    Keyword::SELECT => Ok(Expression::SubQuery(Box::new(self.parse_select()?))),
                    _ => self.make_error(format_args!("invalid keyword {kw} expect expression")),
                },
                Token::Identifier(_) => {
                    let ident: Identifier = self.parse_identifier()?;
                    // parse function starts with '(' and, loop arguments until ')'
                    if self.next_if(|t| *t == Token::LeftParenthesis) {
                        let mut args: Vec<Expression> = Vec::new();
                        while !self.next_if(|t| *t == Token::RightParenthesis) {
                            args.push(self.parse_expression(0)?);
                            self.next_if(|t| *t == Token::Comma);
                        }
                        Ok(Expression::Function(Function {
                            name: ident,
                            args: args.into_boxed_slice(),
                        }))
                    } else {
                        Ok(Expression::Identifier(ident))
                    }
                }
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
                Token::Multiply => {
                    let expr: Expression =
                        Expression::Identifier(Identifier::Wildcard(Leaf::new(&token.location)));
                    self.next(); // consume *
                    Ok(expr)
                }
                Token::LeftParenthesis => {
                    self.next(); // consume
                    let expr: Expression = self.parse_expression(0)?;
                    match self.peek() {
                        Some(token) => match token.token {
                            Token::RightParenthesis => {
                                self.next(); // consume
                                Ok(expr)
                            }
                            _ => self.make_error(format_args!("invalid token {token}, expect )")),
                        },
                        None => self.make_error(format_args!("unexpected end of input, expect )")),
                    }
                }
                _ => self.make_error(format_args!("invalid token {token}, expect expression")),
            },
            None => self.make_error(format_args!("unexpected end of input, expect select-item")),
        }
    }

    fn peek_binary_operator(&mut self) -> Result<Option<BinaryOperator>, ParseError> {
        match self.peek() {
            Some(token) => match token.token {
                Token::Plus => Ok(Some(BinaryOperator::Plus(Leaf::new(self.location())))),
                Token::Minus => Ok(Some(BinaryOperator::Minus(Leaf::new(self.location())))),
                Token::Multiply => Ok(Some(BinaryOperator::Multiply(Leaf::new(self.location())))),
                Token::Divide => Ok(Some(BinaryOperator::Divide(Leaf::new(self.location())))),
                Token::Equal => Ok(Some(BinaryOperator::Equal(Leaf::new(self.location())))),
                Token::NotEqual => Ok(Some(BinaryOperator::NotEqual(Leaf::new(self.location())))),
                Token::LessThan => Ok(Some(BinaryOperator::LessThan(Leaf::new(self.location())))),
                Token::GreaterThan => Ok(Some(BinaryOperator::GreaterThan(Leaf::new(
                    self.location(),
                )))),
                Token::LessThanOrEqual => Ok(Some(BinaryOperator::LessThanOrEqual(Leaf::new(
                    self.location(),
                )))),
                Token::GreaterThanOrEqual => Ok(Some(BinaryOperator::GreaterThanOrEqual(
                    Leaf::new(self.location()),
                ))),
                Token::Keyword(Keyword::AND) => {
                    Ok(Some(BinaryOperator::AND(Leaf::new(self.location()))))
                }
                Token::Keyword(Keyword::OR) => {
                    Ok(Some(BinaryOperator::OR(Leaf::new(self.location()))))
                }
                _ => Ok(None),
            },
            None => Ok(None),
        }
    }

    fn parse_number(&mut self, _zeros: u16, number: Option<u64>) -> Result<Literal, ParseError> {
        debug_assert_eq!(
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
                        Ok(Identifier::Single(identifiers.pop().unwrap()))
                    } else {
                        Ok(Identifier::Combined(identifiers.into_boxed_slice()))
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
        debug_assert_eq!(self.peek().unwrap().token, Token::Semicolon);
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
        debug_assert!(self.index <= self.tokens.len());
        self.index += 1;
    }

    fn location(&self) -> &TokenLocation {
        let len: usize = self.tokens.len();
        if self.index < len {
            &self.tokens[self.index].location
        } else {
            &self.tokens[len - 1].location
        }
    }

    fn location_and_next(&mut self) -> TokenLocation {
        let loc = self.location().clone();
        self.next();
        loc
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
        let tokens: ParsedTokens = Tokenizer::new(";").tokenize().unwrap();
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
        let tokens: ParsedTokens = Tokenizer::new("SELECT a;").tokenize().unwrap();
        let mut parser: Parser<'_> = Parser::new(&tokens);
        let statements: Statements = parser.parse().unwrap();
        // println!("{:?}", statements);
        assert_eq!(statements.statements.len(), 1);
        assert_eq!(
            statements.statements[0],
            Statement::Select(Select {
                items: vec![SelectItem::Expression(Expression::Identifier(
                    Identifier::Single(SingleIdentifier {
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
                having: None,
            })
        )
    }

    #[test]
    fn select_abc_abc_def() {
        let tokens: ParsedTokens = Tokenizer::new("SELECT abcABCdef;").tokenize().unwrap();
        let mut parser: Parser<'_> = Parser::new(&tokens);
        let statements: Statements = parser.parse().unwrap();
        // println!("{:?}", statements);
        assert_eq!(statements.statements.len(), 1);
        assert_eq!(
            statements.statements[0],
            Statement::Select(Select {
                items: vec![SelectItem::Expression(Expression::Identifier(
                    Identifier::Single(SingleIdentifier {
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
                having: None,
            })
        )
    }

    #[test]
    fn select_wildcard() {
        let tokens: ParsedTokens = Tokenizer::new("SELECT *").tokenize().unwrap();
        let mut parser: Parser<'_> = Parser::new(&tokens);
        let statements: Statements = parser.parse().unwrap();
        // println!("{:?}", statements);
        assert_eq!(statements.statements.len(), 1);
        assert_eq!(
            statements.statements[0],
            Statement::Select(Select {
                items: vec![SelectItem::Expression(Expression::Identifier(
                    Identifier::Wildcard(Leaf::new(&tokens.tokens[1].location))
                ))]
                .into_boxed_slice(),
                from: vec![].into_boxed_slice(),
                wheres: None,
                order_by: vec![].into_boxed_slice(),
                group_by: vec![].into_boxed_slice(),
                limit: None,
                offset: None,
                having: None,
            })
        )
    }

    #[test]
    fn select_abc_abc_def_dot() {
        let tokens: ParsedTokens = Tokenizer::new("SELECT abc.ABC.def;").tokenize().unwrap();
        let mut parser: Parser<'_> = Parser::new(&tokens);
        let statements: Statements = parser.parse().unwrap();
        // println!("{:?}", statements);
        assert_eq!(statements.statements.len(), 1);
        assert_eq!(
            statements.statements[0],
            Statement::Select(Select {
                items: vec![SelectItem::Expression(Expression::Identifier(
                    Identifier::Combined(
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
                having: None,
            })
        )
    }

    #[test]
    fn select_many() {
        let tokens: ParsedTokens = Tokenizer::new(" SELECT    a, * ,  b.c ;  ")
            .tokenize()
            .unwrap();
        println!("{:#?}", tokens);
        let mut parser: Parser<'_> = Parser::new(&tokens);
        let statements: Statements = parser.parse().unwrap();
        // println!("{:?}", statements);
        assert_eq!(statements.statements.len(), 1);
        assert_eq!(
            statements.statements[0],
            Statement::Select(Select {
                items: vec![
                    SelectItem::Expression(Expression::Identifier(Identifier::Single(
                        SingleIdentifier {
                            value: "a".into(),
                            leaf: Leaf::new(&tokens.tokens[1].location),
                        }
                    ))),
                    SelectItem::Expression(Expression::Identifier(Identifier::Wildcard(
                        Leaf::new(&tokens.tokens[3].location)
                    ))),
                    SelectItem::Expression(Expression::Identifier(Identifier::Combined(
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
                having: None,
            })
        )
    }

    #[test]
    fn select_str() {
        let tokens: ParsedTokens = Tokenizer::new("SELECT 'hello'").tokenize().unwrap();
        let mut parser: Parser<'_> = Parser::new(&tokens);
        let statements: Statements = parser.parse().unwrap();
        // println!("{:?}", statements);
        assert_eq!(statements.statements.len(), 1);
        assert_eq!(
            statements.statements[0],
            Statement::Select(Select {
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
                having: None,
            })
        )
    }

    #[test]
    fn select_str2() {
        let tokens: ParsedTokens = Tokenizer::new("SELECT 'hello', 'world!\\n';")
            .tokenize()
            .unwrap();
        let mut parser: Parser<'_> = Parser::new(&tokens);
        let statements: Statements = parser.parse().unwrap();
        // println!("{:?}", statements);
        assert_eq!(statements.statements.len(), 1);
        assert_eq!(
            statements.statements[0],
            Statement::Select(Select {
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
                having: None,
            })
        )
    }

    #[test]
    fn select_int() {
        let tokens: ParsedTokens = Tokenizer::new("SELECT 0, 00, 123, 001100;")
            .tokenize()
            .unwrap();
        let mut parser: Parser<'_> = Parser::new(&tokens);
        let statements: Statements = parser.parse().unwrap();
        // println!("{:?}", statements);
        assert_eq!(statements.statements.len(), 1);
        assert_eq!(
            statements.statements[0],
            Statement::Select(Select {
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
                having: None,
            })
        )
    }

    #[test]
    fn select_float() {
        let tokens: ParsedTokens = Tokenizer::new("SELECT 1.0, 1.25, 0.625, 3.0625")
            .tokenize()
            .unwrap();
        let mut parser: Parser<'_> = Parser::new(&tokens);
        let statements: Statements = parser.parse().unwrap();
        // println!("{:?}", statements);
        assert_eq!(statements.statements.len(), 1);
        assert_eq!(
            statements.statements[0],
            Statement::Select(Select {
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
                having: None,
            })
        )
    }

    #[test]
    fn select_int_int_float() {
        let tokens: ParsedTokens = Tokenizer::new("SELECT 1.0, 1.25, 0.625, 3.0625, 123")
            .tokenize()
            .unwrap();
        let mut parser: Parser<'_> = Parser::new(&tokens);
        let statements: Statements = parser.parse().unwrap();
        // println!("{:?}", statements);
        assert_eq!(statements.statements.len(), 1);
        assert_eq!(
            statements.statements[0],
            Statement::Select(Select {
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
                having: None,
            })
        )
    }

    #[test]
    fn test_one_add_two() {
        let tokens: ParsedTokens = Tokenizer::new("SELECT 1+2;").tokenize().unwrap();
        let mut parser: Parser<'_> = Parser::new(&tokens);
        let statements: Statements = parser.parse().unwrap();
        assert_eq!(statements.statements.len(), 1);
        println!("{:}", tokens);
        println!("{:}", statements.statements[0]);
        assert_eq!(
            statements.statements[0],
            Statement::Select(Select {
                items: vec![SelectItem::Expression(Expression::BinaryExpression(
                    BinaryExpression {
                        left: Box::new(Expression::Literal(Literal {
                            value: Value::Integer(1),
                            leaf: Leaf::new(&tokens.tokens[1].location)
                        })),
                        right: Box::new(Expression::Literal(Literal {
                            value: Value::Integer(2),
                            leaf: Leaf::new(&tokens.tokens[3].location)
                        })),
                        operator: BinaryOperator::Plus(Leaf::new(&tokens.tokens[2].location)),
                    }
                ))]
                .into_boxed_slice(),
                from: vec![].into_boxed_slice(),
                wheres: None,
                order_by: vec![].into_boxed_slice(),
                group_by: vec![].into_boxed_slice(),
                limit: None,
                offset: None,
                having: None,
            })
        );
    }

    #[test]
    fn test_one_add_two_minus_three() {
        let tokens: ParsedTokens = Tokenizer::new("SELECT 1+2-3;").tokenize().unwrap();
        let mut parser: Parser<'_> = Parser::new(&tokens);
        let statements: Statements = parser.parse().unwrap();
        assert_eq!(statements.statements.len(), 1);
        println!("{:}", tokens);
        println!("{:}", statements.statements[0]);
        assert_eq!(
            statements.statements[0],
            Statement::Select(Select {
                items: vec![SelectItem::Expression(Expression::BinaryExpression(
                    BinaryExpression {
                        left: Box::new(Expression::BinaryExpression(BinaryExpression {
                            left: Box::new(Expression::Literal(Literal {
                                value: Value::Integer(1),
                                leaf: Leaf::new(&tokens.tokens[1].location)
                            })),
                            right: Box::new(Expression::Literal(Literal {
                                value: Value::Integer(2),
                                leaf: Leaf::new(&tokens.tokens[3].location)
                            })),
                            operator: BinaryOperator::Plus(Leaf::new(&tokens.tokens[2].location)),
                        })),
                        right: Box::new(Expression::Literal(Literal {
                            value: Value::Integer(3),
                            leaf: Leaf::new(&tokens.tokens[5].location),
                        })),
                        operator: BinaryOperator::Minus(Leaf::new(&tokens.tokens[4].location)),
                    }
                ))]
                .into_boxed_slice(),
                from: vec![].into_boxed_slice(),
                wheres: None,
                order_by: vec![].into_boxed_slice(),
                group_by: vec![].into_boxed_slice(),
                limit: None,
                offset: None,
                having: None,
            })
        );
    }

    #[test]
    fn test_one_add_two_mul_three() {
        let tokens: ParsedTokens = Tokenizer::new("SELECT 1+2*3;").tokenize().unwrap();
        let mut parser: Parser<'_> = Parser::new(&tokens);
        let statements: Statements = parser.parse().unwrap();
        assert_eq!(statements.statements.len(), 1);
        println!("{:}", tokens);
        println!("{:}", statements.statements[0]);
        assert_eq!(
            statements.statements[0],
            Statement::Select(Select {
                items: vec![SelectItem::Expression(Expression::BinaryExpression(
                    BinaryExpression {
                        left: Box::new(Expression::Literal(Literal {
                            value: Value::Integer(1),
                            leaf: Leaf::new(&tokens.tokens[1].location)
                        })),
                        right: Box::new(Expression::BinaryExpression(BinaryExpression {
                            left: Box::new(Expression::Literal(Literal {
                                value: Value::Integer(2),
                                leaf: Leaf::new(&tokens.tokens[3].location)
                            })),
                            right: Box::new(Expression::Literal(Literal {
                                value: Value::Integer(3),
                                leaf: Leaf::new(&tokens.tokens[5].location)
                            })),
                            operator: BinaryOperator::Multiply(Leaf::new(
                                &tokens.tokens[4].location
                            )),
                        })),
                        operator: BinaryOperator::Plus(Leaf::new(&tokens.tokens[2].location)),
                    }
                ))]
                .into_boxed_slice(),
                from: vec![].into_boxed_slice(),
                wheres: None,
                order_by: vec![].into_boxed_slice(),
                group_by: vec![].into_boxed_slice(),
                limit: None,
                offset: None,
                having: None,
            })
        );
    }

    #[test]
    fn test_paren_1() {
        let tokens: ParsedTokens = Tokenizer::new("SELECT (1);").tokenize().unwrap();
        let mut parser: Parser<'_> = Parser::new(&tokens);
        let statements: Statements = parser.parse().unwrap();
        assert_eq!(statements.statements.len(), 1);
        println!("{:}", tokens);
        println!("{:}", statements.statements[0]);
        assert_eq!(
            statements.statements[0],
            Statement::Select(Select {
                items: vec![SelectItem::Expression(Expression::Literal(Literal {
                    value: Value::Integer(1),
                    leaf: Leaf::new(&tokens.tokens[2].location)
                }))]
                .into_boxed_slice(),
                from: vec![].into_boxed_slice(),
                wheres: None,
                order_by: vec![].into_boxed_slice(),
                group_by: vec![].into_boxed_slice(),
                limit: None,
                offset: None,
                having: None,
            })
        );
    }

    #[test]
    fn test_paren_paren_1() {
        let tokens: ParsedTokens = Tokenizer::new("SELECT ((1));").tokenize().unwrap();
        let mut parser: Parser<'_> = Parser::new(&tokens);
        let statements: Statements = parser.parse().unwrap();
        assert_eq!(statements.statements.len(), 1);
        println!("{:}", tokens);
        println!("{:}", statements.statements[0]);
        assert_eq!(
            statements.statements[0],
            Statement::Select(Select {
                items: vec![SelectItem::Expression(Expression::Literal(Literal {
                    value: Value::Integer(1),
                    leaf: Leaf::new(&tokens.tokens[3].location)
                }))]
                .into_boxed_slice(),
                from: vec![].into_boxed_slice(),
                wheres: None,
                order_by: vec![].into_boxed_slice(),
                group_by: vec![].into_boxed_slice(),
                limit: None,
                offset: None,
                having: None,
            })
        );
    }

    #[test]
    fn test_2_mul_sum_of_3_and_4() {
        let tokens: ParsedTokens = Tokenizer::new("SELECT 2*(3+4);").tokenize().unwrap();
        let mut parser: Parser<'_> = Parser::new(&tokens);
        let statements: Statements = parser.parse().unwrap();
        assert_eq!(statements.statements.len(), 1);
        println!("{:}", tokens);
        println!("{:}", statements.statements[0]);
        assert_eq!(
            statements.statements[0],
            Statement::Select(Select {
                items: vec![SelectItem::Expression(Expression::BinaryExpression(
                    BinaryExpression {
                        left: Box::new(Expression::Literal(Literal {
                            value: Value::Integer(2),
                            leaf: Leaf::new(&tokens.tokens[1].location)
                        })),
                        right: Box::new(Expression::BinaryExpression(BinaryExpression {
                            left: Box::new(Expression::Literal(Literal {
                                value: Value::Integer(3),
                                leaf: Leaf::new(&tokens.tokens[4].location)
                            })),
                            right: Box::new(Expression::Literal(Literal {
                                value: Value::Integer(4),
                                leaf: Leaf::new(&tokens.tokens[6].location)
                            })),
                            operator: BinaryOperator::Plus(Leaf::new(&tokens.tokens[5].location)),
                        })),
                        operator: BinaryOperator::Multiply(Leaf::new(&tokens.tokens[2].location)),
                    }
                ))]
                .into_boxed_slice(),
                from: vec![].into_boxed_slice(),
                wheres: None,
                order_by: vec![].into_boxed_slice(),
                group_by: vec![].into_boxed_slice(),
                limit: None,
                offset: None,
                having: None,
            })
        );
    }

    #[test]
    fn test_function_call() {
        let tokens: ParsedTokens = Tokenizer::new("SELECT foo();").tokenize().unwrap();
        let mut parser: Parser<'_> = Parser::new(&tokens);
        let statements: Statements = parser.parse().unwrap();
        assert_eq!(statements.statements.len(), 1);
        println!("{:}", tokens);
        println!("{:}", statements.statements[0]);
        assert_eq!(
            statements.statements[0],
            Statement::Select(Select {
                items: vec![SelectItem::Expression(Expression::Function(Function {
                    name: Identifier::Single(SingleIdentifier {
                        value: "foo".into(),
                        leaf: Leaf::new(&tokens.tokens[1].location)
                    }),
                    args: vec![].into_boxed_slice(),
                }))]
                .into_boxed_slice(),
                from: vec![].into_boxed_slice(),
                wheres: None,
                order_by: vec![].into_boxed_slice(),
                group_by: vec![].into_boxed_slice(),
                limit: None,
                offset: None,
                having: None,
            })
        );
    }

    #[test]
    fn test_function_call_1() {
        let tokens: ParsedTokens = Tokenizer::new("SELECT foo(1);").tokenize().unwrap();
        let mut parser: Parser<'_> = Parser::new(&tokens);
        let statements: Statements = parser.parse().unwrap();
        assert_eq!(statements.statements.len(), 1);
        println!("{:}", tokens);
        println!("{:}", statements.statements[0]);
        assert_eq!(
            statements.statements[0],
            Statement::Select(Select {
                items: vec![SelectItem::Expression(Expression::Function(Function {
                    name: Identifier::Single(SingleIdentifier {
                        value: "foo".into(),
                        leaf: Leaf::new(&tokens.tokens[1].location)
                    }),
                    args: vec![Expression::Literal(Literal {
                        value: Value::Integer(1),
                        leaf: Leaf::new(&tokens.tokens[3].location)
                    })]
                    .into_boxed_slice(),
                }))]
                .into_boxed_slice(),
                from: vec![].into_boxed_slice(),
                wheres: None,
                order_by: vec![].into_boxed_slice(),
                group_by: vec![].into_boxed_slice(),
                limit: None,
                offset: None,
                having: None,
            })
        );
    }

    #[test]
    fn test_function_call_1_a_b() {
        let tokens: ParsedTokens = Tokenizer::new("SELECT foo(1, a.b);").tokenize().unwrap();
        let mut parser: Parser<'_> = Parser::new(&tokens);
        let statements: Statements = parser.parse().unwrap();
        assert_eq!(statements.statements.len(), 1);
        println!("{:}", tokens);
        println!("{:}", statements.statements[0]);
        assert_eq!(
            statements.statements[0],
            Statement::Select(Select {
                items: vec![SelectItem::Expression(Expression::Function(Function {
                    name: Identifier::Single(SingleIdentifier {
                        value: "foo".into(),
                        leaf: Leaf::new(&tokens.tokens[1].location)
                    }),
                    args: vec![
                        Expression::Literal(Literal {
                            value: Value::Integer(1),
                            leaf: Leaf::new(&tokens.tokens[3].location)
                        }),
                        Expression::Identifier(Identifier::Combined(
                            vec![
                                SingleIdentifier {
                                    value: "a".into(),
                                    leaf: Leaf::new(&tokens.tokens[5].location)
                                },
                                SingleIdentifier {
                                    value: "b".into(),
                                    leaf: Leaf::new(&tokens.tokens[7].location)
                                },
                            ]
                            .into_boxed_slice()
                        ))
                    ]
                    .into_boxed_slice(),
                }))]
                .into_boxed_slice(),
                from: vec![].into_boxed_slice(),
                wheres: None,
                order_by: vec![].into_boxed_slice(),
                group_by: vec![].into_boxed_slice(),
                limit: None,
                offset: None,
                having: None,
            })
        );
    }

    #[test]
    fn test_function_call_1_a_add_b() {
        let tokens: ParsedTokens = Tokenizer::new("SELECT foo(1, a+b);").tokenize().unwrap();
        let mut parser: Parser<'_> = Parser::new(&tokens);
        let statements: Statements = parser.parse().unwrap();
        assert_eq!(statements.statements.len(), 1);
        println!("{:}", tokens);
        println!("{:}", statements.statements[0]);
        assert_eq!(
            statements.statements[0],
            Statement::Select(Select {
                items: vec![SelectItem::Expression(Expression::Function(Function {
                    name: Identifier::Single(SingleIdentifier {
                        value: "foo".into(),
                        leaf: Leaf::new(&tokens.tokens[1].location)
                    }),
                    args: vec![
                        Expression::Literal(Literal {
                            value: Value::Integer(1),
                            leaf: Leaf::new(&tokens.tokens[3].location)
                        }),
                        Expression::BinaryExpression(BinaryExpression {
                            left: Box::new(Expression::Identifier(Identifier::Single(
                                SingleIdentifier {
                                    value: "a".into(),
                                    leaf: Leaf::new(&tokens.tokens[5].location)
                                }
                            ))),
                            right: Box::new(Expression::Identifier(Identifier::Single(
                                SingleIdentifier {
                                    value: "b".into(),
                                    leaf: Leaf::new(&tokens.tokens[7].location)
                                }
                            ))),
                            operator: BinaryOperator::Plus(Leaf::new(&tokens.tokens[6].location)),
                        })
                    ]
                    .into_boxed_slice(),
                }))]
                .into_boxed_slice(),
                from: vec![].into_boxed_slice(),
                wheres: None,
                order_by: vec![].into_boxed_slice(),
                group_by: vec![].into_boxed_slice(),
                limit: None,
                offset: None,
                having: None,
            })
        );
    }

    #[test]
    fn test_select_a_from_b() {
        let tokens: ParsedTokens = Tokenizer::new("SELECT a FROM b;").tokenize().unwrap();
        let mut parser: Parser<'_> = Parser::new(&tokens);
        let statements: Statements = parser.parse().unwrap();
        assert_eq!(statements.statements.len(), 1);
        println!("{:}", tokens);
        println!("{:}", statements.statements[0]);
        assert_eq!(
            statements.statements[0],
            Statement::Select(Select {
                items: vec![SelectItem::Expression(Expression::Identifier(
                    Identifier::Single(SingleIdentifier {
                        value: "a".into(),
                        leaf: Leaf::new(&tokens.tokens[1].location)
                    })
                ))]
                .into_boxed_slice(),
                from: vec![FromItem {
                    expression: Expression::Identifier(Identifier::Single(SingleIdentifier {
                        value: "b".into(),
                        leaf: Leaf::new(&tokens.tokens[3].location)
                    })),
                    alias: None
                }]
                .into_boxed_slice(),
                wheres: None,
                order_by: vec![].into_boxed_slice(),
                group_by: vec![].into_boxed_slice(),
                limit: None,
                offset: None,
                having: None,
            })
        );
    }

    #[test]
    fn test_select_a_from_c_d_as_e() {
        let tokens: ParsedTokens = Tokenizer::new("SELECT a FROM c, d AS e;")
            .tokenize()
            .unwrap();
        let mut parser: Parser<'_> = Parser::new(&tokens);
        let statements: Statements = parser.parse().unwrap();
        assert_eq!(statements.statements.len(), 1);
        println!("{:}", tokens);
        println!("{:}", statements.statements[0]);
        assert_eq!(
            statements.statements[0],
            Statement::Select(Select {
                items: vec![SelectItem::Expression(Expression::Identifier(
                    Identifier::Single(SingleIdentifier {
                        value: "a".into(),
                        leaf: Leaf::new(&tokens.tokens[1].location)
                    })
                ))]
                .into_boxed_slice(),
                from: vec![
                    FromItem {
                        expression: Expression::Identifier(Identifier::Single(SingleIdentifier {
                            value: "c".into(),
                            leaf: Leaf::new(&tokens.tokens[3].location)
                        })),
                        alias: None
                    },
                    FromItem {
                        expression: Expression::Identifier(Identifier::Single(SingleIdentifier {
                            value: "d".into(),
                            leaf: Leaf::new(&tokens.tokens[5].location)
                        })),
                        alias: Some(Identifier::Single(SingleIdentifier {
                            value: "e".into(),
                            leaf: Leaf::new(&tokens.tokens[7].location)
                        }))
                    }
                ]
                .into_boxed_slice(),
                wheres: None,
                order_by: vec![].into_boxed_slice(),
                group_by: vec![].into_boxed_slice(),
                limit: None,
                offset: None,
                having: None,
            })
        );
    }

    #[test]
    fn test_select_a_from_b_where_c() {
        let tokens: ParsedTokens = Tokenizer::new("SELECT a FROM b WHERE c>1;")
            .tokenize()
            .unwrap();
        let mut parser: Parser<'_> = Parser::new(&tokens);
        let statements: Statements = parser.parse().unwrap();
        assert_eq!(statements.statements.len(), 1);
        println!("{:}", tokens);
        println!("{:}", statements.statements[0]);
        assert_eq!(
            statements.statements[0],
            Statement::Select(Select {
                items: vec![SelectItem::Expression(Expression::Identifier(
                    Identifier::Single(SingleIdentifier {
                        value: "a".into(),
                        leaf: Leaf::new(&tokens.tokens[1].location)
                    })
                ))]
                .into_boxed_slice(),
                from: vec![FromItem {
                    expression: Expression::Identifier(Identifier::Single(SingleIdentifier {
                        value: "b".into(),
                        leaf: Leaf::new(&tokens.tokens[3].location)
                    })),
                    alias: None,
                }]
                .into_boxed_slice(),
                wheres: Some(Expression::BinaryExpression(BinaryExpression {
                    left: Box::new(Expression::Identifier(Identifier::Single(
                        SingleIdentifier {
                            value: "c".into(),
                            leaf: Leaf::new(&tokens.tokens[5].location)
                        }
                    ))),
                    right: Box::new(Expression::Literal(Literal {
                        value: Value::Integer(1),
                        leaf: Leaf::new(&tokens.tokens[7].location)
                    })),
                    operator: BinaryOperator::GreaterThan(Leaf::new(&tokens.tokens[6].location)),
                })),
                order_by: vec![].into_boxed_slice(),
                group_by: vec![].into_boxed_slice(),
                limit: None,
                offset: None,
                having: None,
            })
        );
    }

    #[test]
    fn test_select_a_from_b_where_c_group_by_e() {
        let tokens: ParsedTokens = Tokenizer::new("SELECT a FROM b WHERE c>1 GROUP BY e;")
            .tokenize()
            .unwrap();
        let mut parser: Parser<'_> = Parser::new(&tokens);
        let statements: Statements = parser.parse().unwrap();
        assert_eq!(statements.statements.len(), 1);
        println!("{:}", tokens);
        println!("{:}", statements.statements[0]);
        assert_eq!(
            statements.statements[0],
            Statement::Select(Select {
                items: vec![SelectItem::Expression(Expression::Identifier(
                    Identifier::Single(SingleIdentifier {
                        value: "a".into(),
                        leaf: Leaf::new(&tokens.tokens[1].location)
                    })
                ))]
                .into_boxed_slice(),
                from: vec![FromItem {
                    expression: Expression::Identifier(Identifier::Single(SingleIdentifier {
                        value: "b".into(),
                        leaf: Leaf::new(&tokens.tokens[3].location)
                    })),
                    alias: None,
                }]
                .into_boxed_slice(),
                wheres: Some(Expression::BinaryExpression(BinaryExpression {
                    left: Box::new(Expression::Identifier(Identifier::Single(
                        SingleIdentifier {
                            value: "c".into(),
                            leaf: Leaf::new(&tokens.tokens[5].location)
                        }
                    ))),
                    right: Box::new(Expression::Literal(Literal {
                        value: Value::Integer(1),
                        leaf: Leaf::new(&tokens.tokens[7].location)
                    })),
                    operator: BinaryOperator::GreaterThan(Leaf::new(&tokens.tokens[6].location)),
                })),
                order_by: vec![].into_boxed_slice(),
                group_by: vec![Identifier::Single(SingleIdentifier {
                    value: "e".into(),
                    leaf: Leaf::new(&tokens.tokens[10].location)
                })]
                .into_boxed_slice(),
                limit: None,
                offset: None,
                having: None,
            })
        );
    }

    #[test]
    fn test_having() {
        let tokens: ParsedTokens = Tokenizer::new("SELECT count(*) a HAVING a>1;")
            .tokenize()
            .unwrap();
        let mut parser: Parser<'_> = Parser::new(&tokens);
        let statements: Statements = parser.parse().unwrap();
        assert_eq!(statements.statements.len(), 1);
        println!("{:}", tokens);
        println!("{:}", statements.statements[0]);
        assert_eq!(
            statements.statements[0],
            Statement::Select(Select {
                items: vec![SelectItem::Alias(Alias {
                    expression: Expression::Function(Function {
                        name: Identifier::Single(SingleIdentifier {
                            value: "count".into(),
                            leaf: Leaf::new(&tokens.tokens[1].location)
                        }),
                        args: vec![Expression::Identifier(Identifier::Wildcard(Leaf::new(
                            &tokens.tokens[3].location
                        )))]
                        .into_boxed_slice(),
                    }),
                    alias: Identifier::Single(SingleIdentifier {
                        value: "a".into(),
                        leaf: Leaf::new(&tokens.tokens[5].location)
                    })
                })]
                .into_boxed_slice(),
                from: vec![].into_boxed_slice(),
                wheres: None,
                order_by: vec![].into_boxed_slice(),
                group_by: vec![].into_boxed_slice(),
                limit: None,
                offset: None,
                having: Some(Expression::BinaryExpression(BinaryExpression {
                    left: Box::new(Expression::Identifier(Identifier::Single(
                        SingleIdentifier {
                            value: "a".into(),
                            leaf: Leaf::new(&tokens.tokens[7].location)
                        }
                    ))),
                    right: Box::new(Expression::Literal(Literal {
                        value: Value::Integer(1),
                        leaf: Leaf::new(&tokens.tokens[9].location)
                    })),
                    operator: BinaryOperator::GreaterThan(Leaf::new(&tokens.tokens[8].location)),
                })),
            })
        );
    }

    #[test]
    fn test_sub_query() {
        let tokens: ParsedTokens = Tokenizer::new("SELECT t.a FROM (SELECT b from c) as t;")
            .tokenize()
            .unwrap();
        let mut parser: Parser<'_> = Parser::new(&tokens);
        let statements: Statements = parser.parse().unwrap();
        assert_eq!(statements.statements.len(), 1);
        println!("{:}", tokens);
        println!("{:}", statements.statements[0]);
        assert_eq!(
            statements.statements[0],
            Statement::Select(Select {
                items: vec![SelectItem::Expression(Expression::Identifier(
                    Identifier::Combined(
                        vec![
                            SingleIdentifier {
                                value: "t".into(),
                                leaf: Leaf::new(&tokens.tokens[1].location)
                            },
                            SingleIdentifier {
                                value: "a".into(),
                                leaf: Leaf::new(&tokens.tokens[3].location)
                            },
                        ]
                        .into_boxed_slice()
                    )
                ))]
                .into_boxed_slice(),
                from: vec![FromItem {
                    expression: Expression::SubQuery(Box::new(Select {
                        items: vec![SelectItem::Expression(Expression::Identifier(
                            Identifier::Single(SingleIdentifier {
                                value: "b".into(),
                                leaf: Leaf::new(&tokens.tokens[7].location)
                            })
                        ))]
                        .into_boxed_slice(),
                        from: vec![FromItem {
                            expression: Expression::Identifier(Identifier::Single(
                                SingleIdentifier {
                                    value: "c".into(),
                                    leaf: Leaf::new(&tokens.tokens[9].location)
                                }
                            )),
                            alias: None,
                        }]
                        .into_boxed_slice(),
                        wheres: None,
                        order_by: vec![].into_boxed_slice(),
                        group_by: vec![].into_boxed_slice(),
                        limit: None,
                        offset: None,
                        having: None,
                    })),
                    alias: Some(Identifier::Single(SingleIdentifier {
                        value: "t".into(),
                        leaf: Leaf::new(&tokens.tokens[12].location)
                    }))
                }]
                .into_boxed_slice(),
                wheres: None,
                order_by: vec![].into_boxed_slice(),
                group_by: vec![].into_boxed_slice(),
                limit: None,
                offset: None,
                having: None,
            })
        );
    }
}
