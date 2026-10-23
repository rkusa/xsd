#[xsd::all(schema = "tests/xsd/attribute_default.xsd")]
mod schema {}

use pretty_assertions::assert_eq;

#[test]
fn attribute_default() {
    let xml = include_str!("./xsd/attribute_default.xml");
    let expected = schema::Text {
        text: "foobar".to_string(),
        lang: "en".to_string(),
        size: 12,
    };
    assert_eq!(schema::Text::from_xml(xml).unwrap(), expected);

    // attributes with a default are always written
    assert_eq!(
        String::from_utf8_lossy(&expected.to_xml().unwrap()),
        r#"<?xml version="1.0" encoding="UTF-8"?>
<text lang="en" size="12">foobar</text>"#
    );
}

#[test]
fn attribute_default_overridden() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<text lang="de" size="8">foobar</text>"#;
    let expected = schema::Text {
        text: "foobar".to_string(),
        lang: "de".to_string(),
        size: 8,
    };
    assert_eq!(schema::Text::from_xml(xml).unwrap(), expected);
    assert_eq!(String::from_utf8_lossy(&expected.to_xml().unwrap()), xml);
}
