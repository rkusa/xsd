#[xsd::all(schema = "tests/xsd/virtual_type_collision.xsd")]
mod schema {}

use pretty_assertions::assert_eq;

#[test]
fn virtual_type_collision() {
    let xml = include_str!("./xsd/virtual_type_collision.xml");
    let expected = schema::Trace(schema::Path {
        point: vec![
            schema::PathPoint {
                point: schema::PathPointData { x: 1.0, y: 2.0 },
            },
            schema::PathPoint {
                point: schema::PathPointData { x: 3.0, y: 4.0 },
            },
        ],
    });
    assert_eq!(schema::Trace::from_xml(xml).unwrap(), expected);
    assert_eq!(String::from_utf8_lossy(&expected.to_xml().unwrap()), xml);
}
