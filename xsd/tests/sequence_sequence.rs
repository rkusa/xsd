#[xsd::all(schema = "tests/xsd/sequence_sequence.xsd")]
mod schema {}

use pretty_assertions::assert_eq;

#[test]
fn sequence_sequence() {
    let xml = include_str!("./xsd/sequence_sequence.xml");
    let expected = schema::Settings {
        scope: "all".to_string(),
        key_value: schema::SettingsKeyValue {
            key: "alpha".to_string(),
            value: 1,
        },
    };
    assert_eq!(schema::Settings::from_xml(xml).unwrap(), expected);
    assert_eq!(String::from_utf8_lossy(&expected.to_xml().unwrap()), xml);
}
