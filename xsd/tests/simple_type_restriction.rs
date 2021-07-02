#[xsd::all(schema = "tests/xsd/simple_type_restriction.xsd")]
mod schema {}

use pretty_assertions::assert_eq;

#[test]
fn simple_type_restriction() {
    let xml = include_str!("./xsd/simple_type_restriction.xml");
    let expected = schema::Side(schema::SideEnum::Left);
    assert_eq!(schema::Side::from_xml(xml).unwrap(), expected);
    assert_eq!(String::from_utf8_lossy(&expected.to_xml().unwrap()), xml);
}
