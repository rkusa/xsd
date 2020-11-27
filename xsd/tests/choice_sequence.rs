#[xsd::all(schema = "tests/xsd/choice_sequence.xsd")]
mod schema {}

use pretty_assertions::assert_eq;

#[test]
fn choice_sequence() {
    let xml = include_str!("./xsd/choice_sequence.xml");
    let expected = schema::User {
        id: 42,
        name: schema::UserName::Name(schema::UserNameVariant {
            first_name: "Foo".to_string(),
            last_name: "Foo".to_string(),
        }),
    };
    assert_eq!(schema::User::from_xml(xml).unwrap(), expected);
    assert_eq!(String::from_utf8_lossy(&expected.to_xml().unwrap()), xml);
}
