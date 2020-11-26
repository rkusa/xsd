#[xsd::all(schema = "tests/xsd/enum_literal.xsd")]
mod schema {}

use pretty_assertions::assert_eq;

#[test]
fn enum_literal() {
    let xml = include_str!("./xsd/enum_literal.xml");
    let expected = schema::Order {
        state: schema::OrderState::Ready,
    };
    assert_eq!(schema::Order::from_xml(xml).unwrap(), expected);
    assert_eq!(String::from_utf8_lossy(&expected.to_xml().unwrap()), xml);
}
