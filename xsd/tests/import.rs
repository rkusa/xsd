#[xsd::all(schema = "tests/xsd/import.xsd")]
mod schema {}

use pretty_assertions::assert_eq;

#[test]
fn import() {
    let xml = include_str!("./xsd/import.xml");
    let expected = schema::Article {
        content: schema::ArticleContent {
            excerpt: "Lorem Ipsum".to_string(),
            author: schema::ArticleContentAuthorData {
                name: "Foobar".to_string(),
            },
        },
    };
    assert_eq!(schema::Article::from_xml(xml).unwrap(), expected);
    assert_eq!(String::from_utf8_lossy(&expected.to_xml().unwrap()), xml);
}
