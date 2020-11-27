#[xsd::all(schema = "tests/xsd/element_default.xsd")]
mod schema {}

use pretty_assertions::assert_eq;

#[test]
fn element_default() {
    let xml = include_str!("./xsd/element_default.xml");
    let expected = schema::Settings { debug: None };
    assert_eq!(schema::Settings::from_xml(xml).unwrap(), expected);
    assert_eq!(String::from_utf8_lossy(&expected.to_xml().unwrap()), xml);
}
