#[xsd::all(schema = "tests/xsd/element_optional_vec.xsd")]
mod schema {}

use pretty_assertions::assert_eq;

#[test]
fn element_optional_vec() {
    let xml = include_str!("./xsd/element_optional_vec.xml");
    let expected = schema::List {
        id: Some(vec![1, 2, 3]),
    };
    assert_eq!(schema::List::from_xml(xml).unwrap(), expected);
    assert_eq!(String::from_utf8_lossy(&expected.to_xml().unwrap()), xml);
}
