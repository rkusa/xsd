#[xsd::all(schema = "tests/xsd/sequence_vec.xsd")]
mod schema {}

use pretty_assertions::assert_eq;

#[test]
fn sequence_vec() {
    let xml = include_str!("./xsd/sequence_vec.xml");
    let expected = schema::Settings {
        key_value: vec![
            schema::SettingsKeyValue {
                key: "alpha".to_string(),
                value: 1,
            },
            schema::SettingsKeyValue {
                key: "bravo".to_string(),
                value: 2,
            },
            schema::SettingsKeyValue {
                key: "charlie".to_string(),
                value: 3,
            },
        ],
    };
    assert_eq!(schema::Settings::from_xml(xml).unwrap(), expected);
    assert_eq!(String::from_utf8_lossy(&expected.to_xml().unwrap()), xml);
}
