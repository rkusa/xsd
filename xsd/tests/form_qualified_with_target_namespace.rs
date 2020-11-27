#[xsd::all(schema = "tests/xsd/form_qualified_with_target_namespace.xsd")]
mod schema {}

use pretty_assertions::assert_eq;

#[test]
fn form_qualified_with_target_namespace() {
    let xml = include_str!("./xsd/form_qualified_with_target_namespace.xml");
    let expected = schema::Article {
        excerpt: "Lorem Ipsum".to_string(),
        author: schema::ArticleAuthorData {
            name: "Foobar".to_string(),
        },
    };
    assert_eq!(schema::Article::from_xml(xml).unwrap(), expected);
    assert_eq!(String::from_utf8_lossy(&expected.to_xml().unwrap()), xml);
}
