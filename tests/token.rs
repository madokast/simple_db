use simple_db::sql::tokenizer::token;

#[test]
fn access_keyword() {
    let kw: token::Keyword = token::Keyword::SELECT;
    assert_eq!(kw, token::Keyword::SELECT);
    assert_eq!(format!("{:?}", kw), "SELECT");
    assert_eq!(format!("{}", kw), "SELECT");
}