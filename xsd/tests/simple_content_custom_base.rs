#[xsd::all(schema = "tests/xsd/simple_content_custom_base.xsd")]
mod schema {}

use pretty_assertions::assert_eq;

#[test]
fn simple_content_custom_base() {
    let xml = include_str!("./xsd/simple_content_custom_base.xml");
    let expected = schema::User {
        role: schema::UserRoleData {
            value: schema::Role::User,
            inherited: true,
        },
    };
    assert_eq!(schema::User::from_xml(xml).unwrap(), expected);
    assert_eq!(String::from_utf8_lossy(&expected.to_xml().unwrap()), xml);
}
