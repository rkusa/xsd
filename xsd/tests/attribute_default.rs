#[xsd::all(schema = "tests/xsd/attribute_default.xsd")]
mod schema {}

use pretty_assertions::assert_eq;

#[test]
fn attribute_default() {
    // missing attributes are decoded as `None` (which is equivalent to their default)
    let xml = include_str!("./xsd/attribute_default.xml");
    let expected = schema::Text {
        text: "foobar".to_string(),
        lang: None,
        size: None,
    };
    assert_eq!(schema::Text::from_xml(xml).unwrap(), expected);
    assert_eq!(String::from_utf8_lossy(&expected.to_xml().unwrap()), xml);
}

#[test]
fn attribute_default_overridden() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<text lang="de" size="8">foobar</text>"#;
    let expected = schema::Text {
        text: "foobar".to_string(),
        lang: Some("de".to_string()),
        size: Some(8),
    };
    assert_eq!(schema::Text::from_xml(xml).unwrap(), expected);
    assert_eq!(String::from_utf8_lossy(&expected.to_xml().unwrap()), xml);
}
