#[xsd::all(schema = "tests/xsd/choice_literal.xsd")]
mod schema {}

use pretty_assertions::assert_eq;

#[test]
fn choice_literal() {
    let xml = include_str!("./xsd/choice_literal.xml");
    let expected = schema::Article {
        user_bot: schema::ArticleUserBot::Bot("foobot".to_string()),
    };
    assert_eq!(schema::Article::from_xml(xml).unwrap(), expected);
    assert_eq!(String::from_utf8_lossy(&expected.to_xml().unwrap()), xml);
}
