use std::fmt::Display;

use super::expression::{Alias, Expression};
use super::identifier::Identifier;
use super::leaf::{Location, WithLocation};

#[derive(Debug, PartialEq, Clone)]
pub struct Select {
    pub items: Box<[SelectItem]>,
    pub from: Box<[FromItem]>,
    pub wheres: Option<Expression>,
    pub group_by: Box<[Identifier]>,
    pub having: Option<Expression>,
    pub order_by: Box<[OrderBy]>,
    pub limit: Option<Limit>,
    pub offset: Option<Offset>,
}

impl WithLocation for Select  {
    fn location(&self) -> &Location {
        self.items[0].location()
    }
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
        if let Some(having) = &self.having {
            write!(f, " HAVING {}", having)?;
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
        Ok(())
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum SelectItem {
    Expression(Expression),
    Alias(Alias),
}

impl WithLocation for SelectItem {
    fn location(&self) -> &Location {
        match self {
            SelectItem::Expression(expression) => expression.location(),
            SelectItem::Alias(a) => a.location(),
        }
    }
}

impl Display for SelectItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SelectItem::Expression(expression) => write!(f, "{}", expression),
            SelectItem::Alias(a) => write!(f, "{}", a),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct FromItem {
    pub expression: Expression,
    pub alias: Option<Identifier>,
}

impl WithLocation for FromItem {
    fn location(&self) -> &Location {
        self.expression.location()
    }
}

impl Display for FromItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.expression)?;
        if let Some(alias) = &self.alias {
            write!(f, " AS {}", alias)?;
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct OrderBy {
    pub identifier: Identifier,
    pub asc: bool,
}

impl WithLocation for OrderBy  {
    fn location(&self) -> &Location {
        self.identifier.location()
    }
}

impl Display for OrderBy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.identifier)?;
        if self.asc {
            write!(f, " ASC")
        } else {
            write!(f, " DESC")
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Limit {
    pub limit: u64,
    pub leaf: Location,
}

impl WithLocation for Limit {
    fn location(&self) -> &Location {
        &self.leaf
    }
}

impl Display for Limit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LIMIT {}", self.limit)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Offset {
    pub offset: u64,
    pub leaf: Location,
}

impl WithLocation for Offset {
    fn location(&self) -> &Location {
        &self.leaf
    }
}

impl Display for Offset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "OFFSET {}", self.offset)
    }
}
