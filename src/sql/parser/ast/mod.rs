
pub mod select;
pub mod identifier;
pub mod literal;
pub mod leaf;

use leaf::Leaf;
pub use select::Select;

#[derive(Debug, PartialEq, Clone)]
pub struct Statements {
    pub statements: Vec<Statement>,
    pub raw_sql: String,
}
#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Select(Box<Select>),
    CreateTable,
    Empty(Leaf),
}
