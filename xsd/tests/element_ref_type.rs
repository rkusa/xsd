#[xsd::all(schema = "tests/xsd/element_ref_type.xsd")]
mod schema {}

use pretty_assertions::assert_eq;

#[test]
fn element_ref_type() {
    let xml = include_str!("./xsd/element_ref_type.xml");
    let expected = schema::Address {
        zip: "12345".to_string(),
    };
    assert_eq!(schema::Address::from_xml(xml).unwrap(), expected);
    assert_eq!(String::from_utf8_lossy(&expected.to_xml().unwrap()), xml);
}
