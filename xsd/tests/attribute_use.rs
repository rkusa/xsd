#[xsd::all(schema = "tests/xsd/attribute_use.xsd")]
mod schema {}

use pretty_assertions::assert_eq;

#[test]
fn attribute_use() {
    let xml = include_str!("./xsd/attribute_use.xml");
    let expected = schema::Setting {
        key: "foo".to_string(),
        value: Some("bar".to_string()),
    };
    assert_eq!(schema::Setting::from_xml(xml).unwrap(), expected);
    assert_eq!(String::from_utf8_lossy(&expected.to_xml().unwrap()), xml);
}
