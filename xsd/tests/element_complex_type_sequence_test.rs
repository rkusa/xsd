#[xsd::all(schema = "tests/xsd/element_complex_type_sequence_test.xsd")]
mod schema {}

use pretty_assertions::assert_eq;

#[test]
fn element_complex_type_sequence_test() {
    let xml = include_str!("./xsd/element_complex_type_sequence_test.xml");
    let expected = schema::Article {
        subject: "Lorem Ipsum".to_string(),
        body: "Lorem ipsum dolor sit amet.".to_string(),
        author: "Foobar".to_string(),
    };
    assert_eq!(schema::Article::from_xml(xml).unwrap(), expected);
    assert_eq!(String::from_utf8_lossy(&expected.to_xml().unwrap()), xml);

    // TODO: test docs?
}
