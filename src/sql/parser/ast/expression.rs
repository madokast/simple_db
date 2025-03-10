use std::fmt::Display;

use super::{identifier::Identifier, leaf::Leaf, literal::Literal};

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Literal(Literal),                   // 123 "123"
    Identifier(Identifier),             // col1 tab1.col1
    BinaryExpression(BinaryExpression), // 1+2
    UnaryExpression(UnaryExpression),   // -1
    Parenthesized(Box<Expression>),     // (1+2)
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Literal(literal) => write!(f, "{}", literal),
            Expression::Identifier(identifier) => write!(f, "{}", identifier),
            Expression::BinaryExpression(binary_expression) => write!(f, "{}", binary_expression),
            Expression::UnaryExpression(unary_expression) => write!(f, "{}", unary_expression),
            Expression::Parenthesized(expression) => write!(f, "({})", expression),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct BinaryExpression {
    pub left: Box<Expression>,
    pub right: Box<Expression>,
    pub operator: BinaryOperator,
}

impl Display for BinaryExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {}", self.left, self.operator, self.right)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct UnaryExpression {
    pub operator: UnaryOperator,
    pub expression: Box<Expression>,
}

impl Display for UnaryExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.operator, self.expression)
    }
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

impl BinaryOperator {
    pub fn priority(&self) -> usize {
        match self {
            BinaryOperator::Plus(_) => 1000,
            BinaryOperator::Minus(_) => 1000,
            BinaryOperator::Multiply(_) => 1010,
            BinaryOperator::Divide(_) => 1010,
            BinaryOperator::Equal(_) => 100,
            BinaryOperator::NotEqual(_) => 105,
            BinaryOperator::GreaterThan(_) => 110,
            BinaryOperator::LessThan(_) => 110,
            BinaryOperator::GreaterThanOrEqual(_) => 110,
            BinaryOperator::LessThanOrEqual(_) => 110,
            BinaryOperator::AND(_) => 15,
            BinaryOperator::OR(_) => 10,
        }
    }
}

impl Display for BinaryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryOperator::Plus(_) => write!(f, "+"),
            BinaryOperator::Minus(_) => write!(f, "-"),
            BinaryOperator::Multiply(_) => write!(f, "*"),
            BinaryOperator::Divide(_) => write!(f, "/"),
            BinaryOperator::Equal(_) => write!(f, "="),
            BinaryOperator::NotEqual(_) => write!(f, "<>"),
            BinaryOperator::GreaterThan(_) => write!(f, ">"),
            BinaryOperator::LessThan(_) => write!(f, "<"),
            BinaryOperator::GreaterThanOrEqual(_) => write!(f, ">="),
            BinaryOperator::LessThanOrEqual(_) => write!(f, "<="),
            BinaryOperator::AND(_) => write!(f, "AND"),
            BinaryOperator::OR(_) => write!(f, "OR"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum UnaryOperator {
    Plus(Leaf),
    Minus(Leaf),
    NOT(Leaf),
}

impl Display for UnaryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnaryOperator::Plus(_) => write!(f, "+"),
            UnaryOperator::Minus(_) => write!(f, "-"),
            UnaryOperator::NOT(_) => write!(f, "NOT"),
        }
    }
}
