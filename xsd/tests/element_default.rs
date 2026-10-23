#[xsd::all(schema = "tests/xsd/element_default.xsd")]
mod schema {}

use pretty_assertions::assert_eq;

#[test]
fn element_default() {
    // defaults don't apply to absent elements
    let xml = include_str!("./xsd/element_default.xml");
    let expected = schema::Settings {
        debug: None,
        level: None,
        name: None,
    };
    assert_eq!(schema::Settings::from_xml(xml).unwrap(), expected);
    assert_eq!(String::from_utf8_lossy(&expected.to_xml().unwrap()), xml);
}

#[test]
fn element_default_empty() {
    // defaults apply to empty elements
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<settings>
  <debug />
  <level></level>
  <name />
</settings>"#;
    let expected = schema::Settings {
        debug: Some(false),
        level: Some(schema::Level::Warn),
        name: Some("unnamed".to_string()),
    };
    assert_eq!(schema::Settings::from_xml(xml).unwrap(), expected);
}

#[test]
fn element_default_overridden() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<settings>
  <debug>true</debug>
  <level>info</level>
  <name>foobar</name>
</settings>"#;
    let expected = schema::Settings {
        debug: Some(true),
        level: Some(schema::Level::Info),
        name: Some("foobar".to_string()),
    };
    assert_eq!(schema::Settings::from_xml(xml).unwrap(), expected);
    assert_eq!(String::from_utf8_lossy(&expected.to_xml().unwrap()), xml);
}
