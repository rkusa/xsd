#[xsd::all(schema = "tests/xsd/import_diamond.xsd")]
mod schema {}

use pretty_assertions::assert_eq;

#[test]
fn import_diamond() {
    let xml = include_str!("./xsd/import_diamond.xml");
    let content = |excerpt: &str, name: &str| schema::ArticleContent {
        excerpt: excerpt.to_string(),
        // a schema imported more than once is only parsed once, so its anonymous types aren't
        // duplicated (and don't need to be numbered)
        author: schema::Author {
            name: name.to_string(),
        },
    };
    let expected = schema::Article {
        content: content("Lorem Ipsum", "Foobar"),
        meta: schema::Meta {
            related: content("Dolor", "Bazqux"),
        },
    };
    assert_eq!(schema::Article::from_xml(xml).unwrap(), expected);
    assert_eq!(String::from_utf8_lossy(&expected.to_xml().unwrap()), xml);
}
