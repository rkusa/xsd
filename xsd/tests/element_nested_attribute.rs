#[xsd::all(schema = "tests/xsd/element_nested_attribute.xsd")]
mod schema {}

use pretty_assertions::assert_eq;

#[test]
fn element_nested_attribute() {
    let xml = include_str!("./xsd/element_nested_attribute.xml");
    let expected = schema::Article {
        excerpt: "Lorem Ipsum".to_string(),
        author: schema::ArticleAuthor {
            id: Some(42),
            name: "Foobar".to_string(),
        },
    };
    assert_eq!(schema::Article::from_xml(xml).unwrap(), expected);
    assert_eq!(String::from_utf8_lossy(&expected.to_xml().unwrap()), xml);
}
