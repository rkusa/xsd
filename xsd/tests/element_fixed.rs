#[xsd::all(schema = "tests/xsd/element_fixed.xsd")]
mod schema {}

use pretty_assertions::assert_eq;

#[test]
fn element_fixed() {
    let xml = include_str!("./xsd/element_fixed.xml");
    let expected = schema::Name(());
    assert_eq!(schema::Name::from_xml(xml).unwrap(), expected);
    assert_eq!(String::from_utf8_lossy(&expected.to_xml().unwrap()), xml);
}
