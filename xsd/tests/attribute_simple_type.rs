#[xsd::all(schema = "tests/xsd/attribute_simple_type.xsd")]
mod schema {}

use pretty_assertions::assert_eq;

#[test]
fn attribute_simple_type() {
    let xml = include_str!("./xsd/attribute_simple_type.xml");
    let expected = schema::Text {
        value_: "Deutsch".to_string(),
        lang: "DE".to_string(),
    };
    assert_eq!(schema::Text::from_xml(xml).unwrap(), expected);
    assert_eq!(String::from_utf8_lossy(&expected.to_xml().unwrap()), xml);
}
