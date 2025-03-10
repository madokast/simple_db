use simple_db::sql::parser::ast::{Select, Statement};
use simple_db::sql::tokenizer::token;

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
        items: vec![].into_boxed_slice(),
        from: vec![].into_boxed_slice(),
        wheres: None,
        order_by: vec![].into_boxed_slice(),
        group_by: vec![].into_boxed_slice(),
        having: None,
        limit: None,
        offset: None,
    }));
}
