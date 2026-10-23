#[xsd::all(schema = "tests/xsd/all.xsd")]
mod schema {}

use pretty_assertions::assert_eq;

#[test]
fn all() {
    let xml = include_str!("./xsd/all.xml");
    let expected = schema::All {
        first: "first".to_string(),
        optional: Some("optional".to_string()),
        second: "second".to_string(),
        third: "third".to_string(),
    };
    assert_eq!(schema::All::from_xml(xml).unwrap(), expected);

    // The order of the generated XML does not match the input, which is fine, but needs to be
    // adjusted for the comparison here.
    let result = expected.to_xml().unwrap();
    let result = String::from_utf8_lossy(&result);
    let mut result = result.lines().collect::<Vec<_>>();
    result[2..6].rotate_left(2);
    result.swap(4, 5);
    assert_eq!(result.join("\n"), xml);
}
