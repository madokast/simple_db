/// parser 词法分析，从 SQL tokens 序列构建抽象语法树 AST

/// AST 抽象语法树各种节点定义，例如 语句、SELECT、表达式、标识符、字面量等
pub mod ast;

/// error 定义 ParseError 错误类型
pub mod error;


/// parser 实现词法分析
pub mod parser;
