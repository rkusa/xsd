#[xsd::all(schema = "tests/xsd/element_simple_type.xsd")]
mod schema {}

use pretty_assertions::assert_eq;

#[test]
fn element_simple_type() {
    let xml = include_str!("./xsd/element_simple_type.xml");
    let expected = schema::Count { value: 42 };
    assert_eq!(schema::Count::from_xml(xml).unwrap(), expected);
    assert_eq!(String::from_utf8_lossy(&expected.to_xml().unwrap()), xml);
}
