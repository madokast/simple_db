use std::fmt::Display;

use super::expression::{Alias, Expression};
use super::identifier::Identifier;
use super::leaf::Leaf;
use super::literal::Literal;

#[derive(Debug, PartialEq, Clone)]
pub struct Select {
    pub items: Box<[SelectItem]>,
    pub from: Box<[FromItem]>,
    pub wheres: Option<Expression>,
    pub group_by: Box<[Identifier]>,
    pub order_by: Box<[OrderBy]>,
    pub limit: Option<Limit>,
    pub offset: Option<Offset>,
}

impl Display for Select {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SELECT ")?;
        for (index, item) in self.items.iter().enumerate() {
            if index > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", item)?;
        }
        if self.from.len() > 0 {
            write!(f, " FROM ")?;
            for (index, identifier) in self.from.iter().enumerate() {
                if index > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}", identifier)?;
            }
        }
        if let Some(wheres) = &self.wheres {
            write!(f, " WHERE {}", wheres)?;
        }
        if self.group_by.len() > 0 {
            write!(f, " GROUP BY ")?;
            for (index, identifier) in self.group_by.iter().enumerate() {
                if index > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}", identifier)?;
            }
        }
        if self.order_by.len() > 0 {
            write!(f, " ORDER BY ")?;
            for (index, order_by) in self.order_by.iter().enumerate() {
                if index > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}", order_by)?;
            }
        }
        if let Some(limit) = &self.limit {
            write!(f, " LIMIT {}", limit)?;
        }
        if let Some(offset) = &self.offset {
            write!(f, " OFFSET {}", offset)?;
        }
        write!(f, ";")
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum SelectItem {
    Wildcard(Leaf), // *
    Expression(Expression),
    Alias(Alias),
}

impl Display for SelectItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SelectItem::Wildcard(_) => write!(f, "*"),
            SelectItem::Expression(expression) => write!(f, "{}", expression),
            SelectItem::Alias(a) => write!(f, "{}", a),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct FromItem {
    pub identifier: Identifier,
    pub alias: Option<Identifier>,
}

impl Display for FromItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.identifier)?;
        if let Some(alias) = &self.alias {
            write!(f, " AS {}", alias)?;
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct OrderBy {
    pub literal: Literal,
    pub asc: bool,
    pub leaf: Leaf,
}

impl Display for OrderBy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ORDER BY {} {}",
            self.literal,
            if self.asc { "ASC" } else { "DESC" }
        )
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Limit {
    pub limit: u64,
    pub leaf: Leaf,
}

impl Display for Limit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LIMIT {}", self.limit)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Offset {
    pub offset: u64,
    pub leaf: Leaf,
}

impl Display for Offset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "OFFSET {}", self.offset)
    }
}
