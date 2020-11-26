#[xsd::all(schema = "tests/xsd/enum_literal_root.xsd")]
mod schema {}

use pretty_assertions::assert_eq;

#[test]
fn enum_literal_root() {
    let xml = include_str!("./xsd/enum_literal_root.xml");
    let expected = schema::State(schema::StateType::Ready);
    assert_eq!(schema::State::from_xml(xml).unwrap(), expected);
    assert_eq!(String::from_utf8_lossy(&expected.to_xml().unwrap()), xml);
}
