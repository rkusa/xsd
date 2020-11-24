#[xsd::all(schema = "tests/xsd/element_nested_test.xsd")]
mod schema {}

use pretty_assertions::assert_eq;

#[test]
fn element_nested_test() {
    let xml = include_str!("./xsd/element_nested_test.xml");
    let expected = schema::Article {
        excerpt: "Lorem Ipsum".to_string(),
        author: schema::ArticleAuthor {
            name: "Foobar".to_string(),
        },
    };
    assert_eq!(schema::Article::from_xml(xml).unwrap(), expected);
    assert_eq!(String::from_utf8_lossy(&expected.to_xml().unwrap()), xml);

    // TODO: test docs?
}
