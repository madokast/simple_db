use simple_db::sql::tokenizer::token;
use simple_db::sql::parser::ast::{Statement, Select};

#[test]
fn access_keyword() {
    let kw: token::Keyword = token::Keyword::SELECT;
    assert_eq!(kw, token::Keyword::SELECT);
    assert_eq!(format!("{:?}", kw), "SELECT");
    assert_eq!(format!("{}", kw), "SELECT");
}

#[test]
fn access_query() {
    let _s = Statement::Select(Box::new(Select {
        items:vec![],
        from: vec![],
        wheres: None,
        order_by: vec![],
        group_by: vec![],
        limit: None,
        offset: None,
    }));
}