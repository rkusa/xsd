#[xsd::all(schema = "tests/xsd/enum_attribute.xsd")]
mod schema {}

use pretty_assertions::assert_eq;

#[test]
fn enum_attribute() {
    let xml = include_str!("./xsd/enum_attribute.xml");
    let expected = schema::Article {
        state: Some(schema::ArticleState::Published),
    };
    assert_eq!(schema::Article::from_xml(xml).unwrap(), expected);
    assert_eq!(String::from_utf8_lossy(&expected.to_xml().unwrap()), xml);
}
