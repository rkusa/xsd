#[xsd::all(schema = "tests/xsd/choice_optional.xsd")]
mod schema {}

use pretty_assertions::assert_eq;

#[test]
fn choice_optional() {
    let xml = include_str!("./xsd/choice_optional.xml");
    let expected = schema::Article {
        body: "Lorem Ipsum".to_string(),
        user_bot: Some(schema::ArticleUserBot::Bot(schema::Bot {
            handle: "foobot".to_string(),
        })),
    };
    assert_eq!(schema::Article::from_xml(xml).unwrap(), expected);
    assert_eq!(String::from_utf8_lossy(&expected.to_xml().unwrap()), xml);
}
