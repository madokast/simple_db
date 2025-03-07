
pub mod select;
pub mod identifier;
pub mod literal;
pub mod leaf;

pub use select::Select;

pub struct Statements {
    pub statements: Vec<Statement>,
    pub raw_sql: String,
}
pub enum Statement {
    Select(Box<Select>),
    CreateTable,
}
