#[xsd::all(schema = "tests/xsd/complex_type_choice.xsd")]
mod schema {}

use pretty_assertions::assert_eq;

#[test]
fn complex_type_choice() {
    let xml = include_str!("./xsd/complex_type_choice.xml");
    let expected = schema::Article {
        created_by: schema::Author {
            value_: schema::AuthorData::User(schema::User {
                name: "Foobar".to_string(),
            }),
            is_verified: true,
        },
        posted_by: schema::ArticlePostedByData {
            value_: schema::ArticlePostedByDataData::Bot(schema::Bot {
                handle: "Foobot".to_string(),
            }),
            is_verified: false,
        },
    };
    assert_eq!(schema::Article::from_xml(xml).unwrap(), expected);
    assert_eq!(String::from_utf8_lossy(&expected.to_xml().unwrap()), xml);
}
