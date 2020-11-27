#[xsd::all(schema = "tests/xsd/sequence_optional.xsd")]
mod schema {}

use pretty_assertions::assert_eq;

#[test]
fn sequence_optional() {
    let xml = include_str!("./xsd/sequence_optional.xml");
    let expected = schema::Settings {
        scope: "all".to_string(),
        key_value: Some(schema::SettingsKeyValue {
            key: "alpha".to_string(),
            value: 1,
        }),
    };
    assert_eq!(schema::Settings::from_xml(xml).unwrap(), expected);
    assert_eq!(String::from_utf8_lossy(&expected.to_xml().unwrap()), xml);
}
