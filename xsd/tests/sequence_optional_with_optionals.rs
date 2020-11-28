#[xsd::all(schema = "tests/xsd/sequence_optional_with_optionals.xsd")]
mod schema {}

use pretty_assertions::assert_eq;

#[test]
fn sequence_optional_with_optionals() {
    let xml = include_str!("./xsd/sequence_optional_with_optionals.xml");
    let expected = schema::Record {
        patient: Some(schema::Patient {
            base: schema::Account { id: 42 },
            account: Some(schema::PatientAccount {
                full_name: None,
                name: Some(schema::PatientName {
                    first_name: None,
                    last_name: Some("Foobar".to_string()),
                }),
            }),
        }),
    };
    assert_eq!(schema::Record::from_xml(xml).unwrap(), expected);
    assert_eq!(String::from_utf8_lossy(&expected.to_xml().unwrap()), xml);
}
