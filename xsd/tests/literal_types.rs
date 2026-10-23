#[xsd::all(schema = "tests/xsd/literal_types.xsd")]
mod schema {}

use pretty_assertions::assert_eq;

#[test]
fn literal_types() {
    let xml = include_str!("./xsd/literal_types.xml");
    let expected = schema::Values {
        r#break: true,
        r#type: false,
        double: 1.5,
        short: -3,
        unsigned_byte: 255,
        unsigned_int: 4_000_000_000,
        uri: "https://example.com".to_string(),
        self_: Some(42),
    };
    assert_eq!(schema::Values::from_xml(xml).unwrap(), expected);
    assert_eq!(String::from_utf8_lossy(&expected.to_xml().unwrap()), xml);
}

#[test]
fn literal_types_lexical_forms() {
    // booleans may be represented as `1`/`0`, and whitespace around non-string types is
    // collapsed
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<values>
  <break>1</break>
  <type> 0 </type>
  <double> 1.5 </double>
  <short>
    -3
  </short>
  <unsignedByte>255</unsignedByte>
  <unsignedInt>4000000000</unsignedInt>
  <uri>https://example.com</uri>
</values>"#;
    let values = schema::Values::from_xml(xml).unwrap();
    assert!(values.r#break);
    assert!(!values.r#type);
    assert_eq!(values.double, 1.5);
    assert_eq!(values.short, -3);
    assert_eq!(values.self_, None);
}
