#[xsd::all(schema = "tests/xsd/element_type_literal.xsd")]
mod schema {}

use pretty_assertions::assert_eq;

#[test]
fn element_type_literal() {
    let xml = include_str!("./xsd/element_type_literal.xml");
    let expected = schema::Name("Foobar".to_string());
    assert_eq!(schema::Name::from_xml(xml).unwrap(), expected);
    assert_eq!(String::from_utf8_lossy(&expected.to_xml().unwrap()), xml);
}

#[test]
fn element_type_literal_with_comments() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<Name><!-- leading -->Foo<!-- inner -->bar<?pi ignored?></Name>"#;
    assert_eq!(
        schema::Name::from_xml(xml).unwrap(),
        schema::Name("Foobar".to_string())
    );
}

#[test]
fn element_type_literal_with_child_element() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<Name>Foo<b>bar</b></Name>"#;
    assert!(matches!(
        schema::Name::from_xml(xml),
        Err(xsd::decode::FromXmlError::TextExpected { .. })
    ));
}
