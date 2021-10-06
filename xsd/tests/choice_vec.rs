#[xsd::all(schema = "tests/xsd/choice_vec.xsd")]
mod schema {}

use pretty_assertions::assert_eq;

#[test]
fn choice_vec() {
    let xml = include_str!("./xsd/choice_vec.xml");
    let expected = schema::User {
        id: 42,
        alias_email: Some(vec![
            schema::UserAliasEmail::Alias("Foo".to_string()),
            schema::UserAliasEmail::Email("foo@example.org".to_string()),
            schema::UserAliasEmail::Email("42@example.org".to_string()),
        ]),
    };
    assert_eq!(schema::User::from_xml(xml).unwrap(), expected);
    assert_eq!(String::from_utf8_lossy(&expected.to_xml().unwrap()), xml);
}
