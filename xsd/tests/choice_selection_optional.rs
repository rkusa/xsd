#[xsd::all(schema = "tests/xsd/choice_selection_optional.xsd")]
mod schema {}

use pretty_assertions::assert_eq;

#[test]
fn choice_selection_optional() {
    let xml = include_str!("./xsd/choice_selection_optional.xml");
    let expected = schema::Geometry(schema::GeometryType {
        diameter: 12.4,
        thickness: schema::GeometryTypeThickness::Thickness(schema::GeometryTypeThicknessVariant {
            thickness: None,
            thickness_reduction: Some(schema::GeometryTypeThicknessReductionData {
                value_: true,
                reference: "shape".to_string(),
            }),
        }),
    });
    assert_eq!(schema::Geometry::from_xml(xml).unwrap(), expected);
    assert_eq!(String::from_utf8_lossy(&expected.to_xml().unwrap()), xml);
}
