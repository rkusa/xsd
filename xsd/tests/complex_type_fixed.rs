#[xsd::all(schema = "tests/xsd/complex_type_fixed.xsd")]
mod schema {}

use pretty_assertions::assert_eq;

#[test]
fn complex_type_fixed() {
    let xml = include_str!("./xsd/complex_type_fixed.xml");
    let expected = schema::Meta {
        version: (),
        name: Some(()),
    };
    assert_eq!(schema::Meta::from_xml(xml).unwrap(), expected);
    assert_eq!(String::from_utf8_lossy(&expected.to_xml().unwrap()), xml);
}
