#[xsd::all(schema = "tests/xsd/simple_type_empty.xsd")]
mod schema {}

use pretty_assertions::assert_eq;

#[test]
fn simple_type_empty() {
    let xml = include_str!("./xsd/simple_type_empty.xml");
    let expected = schema::Text(String::new());
    assert_eq!(schema::Text::from_xml(xml).unwrap(), expected);
    assert_eq!(String::from_utf8_lossy(&expected.to_xml().unwrap()), xml);
}
