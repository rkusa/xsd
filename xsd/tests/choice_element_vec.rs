#[xsd::all(schema = "tests/xsd/choice_element_vec.xsd")]
mod schema {}

use pretty_assertions::assert_eq;

#[test]
fn choice_element_vec() {
    let xml = include_str!("./xsd/choice_element_vec.xml");
    let expected = schema::Shape {
        s: schema::ShapeS::Points(vec!["1,2".to_string(), "2,3".to_string()]),
    };
    assert_eq!(schema::Shape::from_xml(xml).unwrap(), expected);
    assert_eq!(String::from_utf8_lossy(&expected.to_xml().unwrap()), xml);
}
