#[xsd::all(schema = "tests/xsd/attribute_default.xsd")]
mod schema {}

use pretty_assertions::assert_eq;

#[test]
fn attribute_default() {
    let xml = include_str!("./xsd/attribute_default.xml");
    let expected = schema::Text {
        value: "foobar".to_string(),
        lang: "en".to_string(),
    };
    assert_eq!(schema::Text::from_xml(xml).unwrap(), expected);
    assert_eq!(String::from_utf8_lossy(&expected.to_xml().unwrap()), xml);
}
