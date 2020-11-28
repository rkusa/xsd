#[xsd::all(schema = "tests/xsd/complex_content_sequence_optional.xsd")]
mod schema {}

use pretty_assertions::assert_eq;

#[test]
fn complex_content_sequence_optional() {
    let xml = include_str!("./xsd/complex_content_sequence_optional.xml");
    let expected = schema::Account {
        base: schema::User {
            id: 42,
            name: "Foobar".to_string(),
        },
        user: Some(schema::AccountUser {
            role: schema::Roles::Admin,
            enabled: true,
        }),
    };
    assert_eq!(schema::Account::from_xml(xml).unwrap(), expected);
    assert_eq!(String::from_utf8_lossy(&expected.to_xml().unwrap()), xml);
}
