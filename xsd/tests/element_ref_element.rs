#[xsd::all(schema = "tests/xsd/element_ref_element.xsd")]
mod schema {}

use pretty_assertions::assert_eq;

#[test]
fn element_ref_element() {
    let xml = include_str!("./xsd/element_ref_element.xml");
    let expected = schema::Service {
        version_number: schema::VersionNumber {
            major: 1,
            minor: 10,
            bug: 3,
        },
    };
    assert_eq!(schema::Service::from_xml(xml).unwrap(), expected);
    assert_eq!(String::from_utf8_lossy(&expected.to_xml().unwrap()), xml);
}
