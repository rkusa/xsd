#[xsd::all(schema = "tests/xsd/anonymous_type_names.xsd")]
mod schema {}

use pretty_assertions::assert_eq;

#[test]
fn anonymous_type_names() {
    let xml = include_str!("./xsd/anonymous_type_names.xml");
    let expected = schema::Order {
        // anonymous types are named after their element ...
        customer: schema::Customer {
            // ... and prefixed with their parent's name if the name is ambiguous
            address: schema::CustomerAddress {
                street: "Main St".to_string(),
            },
        },
        shipping: schema::Shipping {
            address: schema::ShippingAddress {
                city: "Springfield".to_string(),
            },
        },
        // ... or collides with a named type
        status: schema::OrderStatus {
            text: "open".to_string(),
        },
        // choices of more than two elements are called `choice`
        choice: schema::OrderChoice::Invoice("INV-1".to_string()),
        choice_2: schema::OrderChoice2::Carrier(schema::OrderCarrier {
            // sequences inside of choices are named after their first element
            carrier: "DHL".to_string(),
            tracking: "123".to_string(),
        }),
        // a choice named like its only element (`TimestampCurrentValue`) has the same path as
        // the element's type, which needs that name too (`CurrentValue` is ambiguous), so the
        // choice falls back to a suffix (instead of both being numbered)
        placed: schema::Timestamp {
            current_value: schema::TimestampCurrentValueChoice::CurrentValue(
                schema::TimestampCurrentValue {
                    current_value: "20261023".to_string(),
                    format: "102".to_string(),
                },
            ),
        },
        validity: schema::Interval {
            current_value: schema::IntervalCurrentValue { days: 30 },
        },
    };
    assert_eq!(schema::Order::from_xml(xml).unwrap(), expected);
    assert_eq!(String::from_utf8_lossy(&expected.to_xml().unwrap()), xml);

    // the named type keeps its name
    let _ = schema::Status { code: 1 };
}
