pub mod expression;
pub mod identifier;
pub mod leaf;
pub mod literal;
pub mod select;

use std::fmt::Display;

use leaf::{Location, WithLocation};
pub use select::Select;

/// Statements SQL 语句，一条 SQL 语句可能包含多条语句
#[derive(Debug, PartialEq, Clone)]
pub struct Statements {
    pub statements: Box<[Statement]>,

    /// 原始 SQL 语句，用于错误提示
    pub raw_sql: Box<str>,
}

/// Statement 定义各种 SQL 语句类型
#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Select(Select),
    CreateTable,
    Empty(Location),
}

impl WithLocation for Statement {
    fn location(&self) -> &Location {
        match self {
            Statement::Select(select) => select.location(),
            Statement::CreateTable => todo!(),
            Statement::Empty(location) => location,
        }
    }
}

impl Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Statement::Select(select) => write!(f, "{};", select),
            Statement::CreateTable => write!(f, "CREATE TABLE"),
            Statement::Empty(_) => write!(f, ";"),
        }
    }
}
