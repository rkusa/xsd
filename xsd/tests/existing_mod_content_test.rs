#[xsd::all(schema = "tests/xsd/element_type_literal_test.xsd")]
mod schema {
    #[derive(Debug, PartialEq)]
    pub struct Profile {
        pub name: Name,
    }
}

#[test]
fn existing_mod_content_test() {
    let _ = schema::Profile {
        name: schema::Name("Foobar".to_string()),
    };
}
