#[xsd::all(schema = "tests/xsd/element_optional.xsd")]
mod schema {}

use pretty_assertions::assert_eq;

#[test]
fn element_optional() {
    let xml = include_str!("./xsd/element_optional.xml");
    let expected = schema::Settings { debug: Some(true) };
    assert_eq!(schema::Settings::from_xml(xml).unwrap(), expected);
    assert_eq!(String::from_utf8_lossy(&expected.to_xml().unwrap()), xml);
}
