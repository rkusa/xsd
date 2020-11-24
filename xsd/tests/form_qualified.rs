#[xsd::all(schema = "tests/xsd/form_qualified.xsd")]
mod schema {}

use pretty_assertions::assert_eq;

#[test]
fn form_qualified() {
    let xml = include_str!("./xsd/form_qualified.xml");
    let expected = schema::Profile {
        name: "Foobar".to_string(),
    };
    assert_eq!(schema::Profile::from_xml(xml).unwrap(), expected);
    assert_eq!(String::from_utf8_lossy(&expected.to_xml().unwrap()), xml);
}
