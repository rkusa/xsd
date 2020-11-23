#[xsd::all(schema = "tests/xsd/element_type_literal_test.xsd")]
mod schema {}

use pretty_assertions::assert_eq;

#[test]
fn element_type_literal_test() {
    let xml = include_str!("./xsd/element_type_literal_test.xml");
    let expected = schema::Name("Foobar".to_string());
    assert_eq!(schema::Name::from_xml(xml).unwrap(), expected);
    assert_eq!(String::from_utf8_lossy(&expected.to_xml().unwrap()), xml)
}
