#[xsd::all(schema = "tests/xsd/all.xsd")]
mod schema {}

use pretty_assertions::assert_eq;

#[test]
fn all() {
    let xml = include_str!("./xsd/all.xml");
    let expected = schema::All {
        first: "first".to_string(),
        second: "second".to_string(),
        third: "third".to_string(),
    };
    assert_eq!(schema::All::from_xml(xml).unwrap(), expected);

    // The order of the generated XML does not match the input, which is fine, but needs to be
    // adjusted for the comparison here.
    let result = expected.to_xml().unwrap();
    let result = String::from_utf8_lossy(&result);
    let mut result = result.lines().collect::<Vec<_>>();
    result.swap(2, 3);
    result.swap(3, 4);
    assert_eq!(result.join("\n"), xml);
}
