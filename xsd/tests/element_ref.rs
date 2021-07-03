#[xsd::all(schema = "tests/xsd/element_ref.xsd")]
mod schema {}

use pretty_assertions::assert_eq;

#[test]
fn element_ref() {
    let xml = include_str!("./xsd/element_ref.xml");
    let expected = schema::Article(schema::ArticleType {
        content: Some(schema::Content(schema::ArticleContent {
            excerpt: "Lorem Ipsum".to_string(),
        })),
        author: schema::Author("Foobar".to_string()),
    });
    assert_eq!(schema::Article::from_xml(xml).unwrap(), expected);
    assert_eq!(String::from_utf8_lossy(&expected.to_xml().unwrap()), xml);
}
