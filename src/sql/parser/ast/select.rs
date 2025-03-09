use super::identifier::Identifier;
use super::leaf::Leaf;
use super::literal::Literal;

#[derive(Debug, PartialEq, Clone)]
pub struct Select {
    pub items: Box<[SelectItem]>,
    pub from: Box<[Identifier]>,
    pub wheres: Option<Expression>,
    pub group_by: Box<[Identifier]>,
    pub order_by: Box<[OrderBy]>,
    pub limit: Option<Limit>,
    pub offset: Option<Offset>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum SelectItem {
    Wildcard(Leaf), // *
    Expression(Expression),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Literal(Literal),       // 123 "123"
    Identifier(Identifier), // col1 tab1.col1
    BinaryExpression(BinaryExpression),
    UnaryExpression(UnaryExpression),
}

#[derive(Debug, PartialEq, Clone)]
pub struct OrderBy {
    pub literal: Literal,
    pub asc: bool,
    pub leaf: Leaf,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Limit {
    pub limit: u64,
    pub leaf: Leaf,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Offset {
    pub offset: u64,
    pub leaf: Leaf,
}

#[derive(Debug, PartialEq, Clone)]
pub struct BinaryExpression {
    pub left: Box<Expression>,
    pub right: Box<Expression>,
    pub operator: BinaryOperator,
}

#[derive(Debug, PartialEq, Clone)]
pub struct UnaryExpression {
    pub operator: UnaryOperator,
    pub expression: Box<Expression>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BinaryOperator {
    Plus(Leaf),
    Minus(Leaf),
    Multiply(Leaf),
    Divide(Leaf),
    Equal(Leaf),
    NotEqual(Leaf),
    GreaterThan(Leaf),
    LessThan(Leaf),
    GreaterThanOrEqual(Leaf),
    LessThanOrEqual(Leaf),
    AND(Leaf),
    OR(Leaf),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum UnaryOperator {
    Plus(Leaf),
    Minus(Leaf),
    NOT(Leaf),
}
