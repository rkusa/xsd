#[xsd::all(schema = "tests/xsd/element_nested.xsd")]
mod schema {}

use pretty_assertions::assert_eq;

#[test]
fn element_nested() {
    let xml = include_str!("./xsd/element_nested.xml");
    let expected = schema::Article {
        excerpt: "Lorem Ipsum".to_string(),
        author: schema::ArticleAuthorData {
            name: "Foobar".to_string(),
        },
    };
    assert_eq!(schema::Article::from_xml(xml).unwrap(), expected);
    assert_eq!(String::from_utf8_lossy(&expected.to_xml().unwrap()), xml);
}
