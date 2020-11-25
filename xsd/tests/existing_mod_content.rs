#[xsd::all(schema = "tests/xsd/element_type_literal.xsd")]
mod schema {
    #[derive(Debug, PartialEq)]
    pub struct Profile {
        pub name: Name,
    }
}

#[test]
fn existing_mod_content() {
    let _ = schema::Profile {
        name: schema::Name {
            value: "Foobar".to_string(),
        },
    };
}
