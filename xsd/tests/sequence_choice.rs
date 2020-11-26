#[xsd::all(schema = "tests/xsd/sequence_choice.xsd")]
mod schema {}

use pretty_assertions::assert_eq;

#[test]
fn sequence_choice() {
    let xml = include_str!("./xsd/sequence_choice.xml");
    let expected = schema::Article {
        body: "Lorem Ipsum".to_string(),
        user_bot: schema::ArticleUserBot::Bot(schema::Bot {
            handle: "foobot".to_string(),
        }),
    };
    assert_eq!(schema::Article::from_xml(xml).unwrap(), expected);
    assert_eq!(String::from_utf8_lossy(&expected.to_xml().unwrap()), xml);
}
