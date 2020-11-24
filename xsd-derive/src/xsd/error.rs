use std::ops::Range;

#[derive(Debug, thiserror::Error)]
pub enum XsdError {
    #[error("XML parsing error: {0}")]
    Xml(#[from] roxmltree::Error),
    // #[error("Error loading import: {0}")]
    // Import(#[from] SchemaError),
    #[error("Error traversing XML tree: {0}")]
    Node(#[from] super::node::NodeError),
    #[error("Encountered unsupported element `{name}` in `{parent}`")]
    UnsupportedElement {
        name: String,
        parent: String,
        range: Range<usize>,
    },
    #[error("Encountered unsupported value `{value}` for attribute `{name}` in `{element}`")]
    UnsupportedAttributeValue {
        name: String,
        value: String,
        element: String,
        range: Range<usize>,
    },
    #[error("Missing element `{name}` in `{parent}`")]
    MissingElement {
        name: String,
        parent: String,
        range: Range<usize>,
    },
    // #[error("Missing attribute `{name}` in `{element}`")]
    // MissingAttribute {
    //     name: String,
    //     element: String,
    //     range: Range<usize>,
    // },
    #[error("Missing namespace for `{prefix}`")]
    MissingNamespace { prefix: String, range: Range<usize> },
    #[error("Multiple types found inside `{name}`")]
    MultipleTypes { name: String, range: Range<usize> },
    // #[error("Failed to parse int value")]
    // ParseInt {
    //     err: std::num::ParseIntError,
    //     range: Range<usize>,
    // },
    // #[error("Failed to parse decimal value")]
    // ParseDecimal {
    //     err: rust_decimal::Error,
    //     range: Range<usize>,
    // },
    #[error("Unsupported XSD type {name}")]
    UnsupportedType { name: String, range: Range<usize> },
    #[error("Encountered circular type while parsing {name}")]
    CircularType { name: String, range: Range<usize> },
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Error on line {row} (offset {col} in {file}): {err}")]
    Xsd {
        #[source]
        err: Box<XsdError>,
        file: String,
        col: u32,
        row: u32,
    },
}

impl XsdError {
    pub fn range(&self) -> Option<&Range<usize>> {
        match self {
            XsdError::Xml(_) => None,
            // XsdError::Import(_) => None,
            XsdError::Node(err) => err.range(),
            XsdError::UnsupportedElement { range, .. } => Some(range),
            XsdError::UnsupportedAttributeValue { range, .. } => Some(range),
            XsdError::MissingElement { range, .. } => Some(range),
            // XsdError::MissingAttribute { range, .. } => Some(range),
            XsdError::MissingNamespace { range, .. } => Some(range),
            XsdError::MultipleTypes { range, .. } => Some(range),
            // XsdError::ParseInt { range, .. } => Some(range),
            // XsdError::ParseDecimal { range, .. } => Some(range),
            XsdError::UnsupportedType { range, .. } => Some(range),
            XsdError::CircularType { range, .. } => Some(range),
        }
    }
}
