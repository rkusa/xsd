#[xsd::all(schema = "tests/xsd/simple_content.xsd")]
mod schema {}

use pretty_assertions::assert_eq;

#[test]
fn simple_content() {
    let xml = include_str!("./xsd/simple_content.xml");
    let expected = schema::Text {
        value_: "Something in English".to_string(),
        lang: Some("en".to_string()),
    };
    assert_eq!(schema::Text::from_xml(xml).unwrap(), expected);
    assert_eq!(String::from_utf8_lossy(&expected.to_xml().unwrap()), xml);
}
