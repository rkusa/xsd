#[xsd::all(schema = "tests/xsd/complex_type_choice.xsd")]
mod schema {}

use pretty_assertions::assert_eq;

#[test]
fn complex_type_choice() {
    let xml = include_str!("./xsd/complex_type_choice.xml");
    let expected = schema::Article {
        author: schema::Author::User(schema::User {
            name: "Foobar".to_string(),
        }),
    };
    assert_eq!(schema::Article::from_xml(xml).unwrap(), expected);
    assert_eq!(String::from_utf8_lossy(&expected.to_xml().unwrap()), xml);
}
