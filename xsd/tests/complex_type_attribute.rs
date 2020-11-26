#[xsd::all(schema = "tests/xsd/complex_type_attribute.xsd")]
mod schema {}

use pretty_assertions::assert_eq;

#[test]
fn complex_type_attribute() {
    let xml = include_str!("./xsd/complex_type_attribute.xml");
    let expected = schema::Article {
        excerpt: "Lorem Ipsum".to_string(),
        author: Some("Foobar".to_string()),
    };
    assert_eq!(schema::Article::from_xml(xml).unwrap(), expected);
    assert_eq!(String::from_utf8_lossy(&expected.to_xml().unwrap()), xml);
}
