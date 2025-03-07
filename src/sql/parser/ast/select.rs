use super::leaf::Leaf;
use super::identifier::Identifier;
use super::literal::Literal;

pub struct Select {
    pub items: Vec<SelectItem>,
    pub from: Vec<Identifier>,
    pub wheres: Option<Expression>,
    pub group_by: Vec<Identifier>,
    pub order_by: Vec<OrderBy>,
    pub limit: Option<Limit>,
    pub offset: Option<Offset>,
}

pub enum SelectItem {
    Wildcard(Leaf), // *
    Identifier(Identifier), // col1
    Literal(Literal), // "123" 123 12.3
}

pub enum Expression {
    Literal(Literal),
    Identifier(Identifier),
    BinaryExpression(BinaryExpression),
    UnaryExpression(UnaryExpression),
}

pub struct OrderBy {
    pub literal: Literal,
    pub asc: bool,
    pub leaf: Leaf,
}

pub struct Limit {
    pub limit: u64,
    pub leaf: Leaf,
}

pub struct Offset {
    pub offset: u64,
    pub leaf: Leaf,
}

pub struct BinaryExpression {
    pub left: Box<Expression>,
    pub right: Box<Expression>,
    pub operator: BinaryOperator,
}

pub struct UnaryExpression {
    pub operator: UnaryOperator,
    pub expression: Box<Expression>,
}

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

pub enum UnaryOperator {
    Plus(Leaf),
    Minus(Leaf),
    NOT(Leaf),
}





