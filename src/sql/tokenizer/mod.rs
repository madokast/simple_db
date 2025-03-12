/// tokenizer 提供 SQL 语句的词法分析

/// token 定义各种 token 类型，包括关键字、标识符、字面量等
pub mod token;

/// str_scanner 提供 SQL 语句的字符串扫描器
/// 在扫描时记录位置信息，用于错误提示
pub mod str_scanner;

/// 定义 TokenizeError
pub mod error;

/// tokenizer 词法分析核心实现
pub mod tokenizer;
