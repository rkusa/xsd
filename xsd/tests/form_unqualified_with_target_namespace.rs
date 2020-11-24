#[xsd::all(schema = "tests/xsd/form_unqualified_with_target_namespace.xsd")]
mod schema {}

use pretty_assertions::assert_eq;

#[test]
fn form_unqualified_with_target_namespace() {
    let xml = include_str!("./xsd/form_unqualified_with_target_namespace.xml");
    let expected = schema::Profile {
        name: "Foobar".to_string(),
    };
    assert_eq!(schema::Profile::from_xml(xml).unwrap(), expected);
    assert_eq!(String::from_utf8_lossy(&expected.to_xml().unwrap()), xml);
}
