#[xsd::all(schema = "tests/xsd/element_ref_default_namespace.xsd")]
mod schema {}

use pretty_assertions::assert_eq;

#[test]
fn element_ref_default_namespace() {
    let xml = include_str!("./xsd/element_ref_default_namespace.xml");
    let expected = schema::Article {
        author: "Foobar".to_string(),
    };
    assert_eq!(schema::Article::from_xml(xml).unwrap(), expected);
    assert_eq!(String::from_utf8_lossy(&expected.to_xml().unwrap()), xml);
}
