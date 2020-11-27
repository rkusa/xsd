#[xsd::all(schema = "tests/xsd/complex_content.xsd")]
mod schema {}

use pretty_assertions::assert_eq;

#[test]
fn complex_content() {
    let xml = include_str!("./xsd/complex_content.xml");
    let expected = schema::Account {
        value: schema::User {
            id: 42,
            name: "Foobar".to_string(),
        },
        role: schema::Roles::Admin,
    };
    assert_eq!(schema::Account::from_xml(xml).unwrap(), expected);
    assert_eq!(String::from_utf8_lossy(&expected.to_xml().unwrap()), xml);
}
