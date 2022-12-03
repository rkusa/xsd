#[derive(Debug, thiserror::Error)]
pub enum XsdError {
    #[error("XML parsing error: {0}")]
    Xml(#[from] roxmltree::Error),
    // #[error("Error loading import: {0}")]
    // Import(#[from] SchemaError),
    #[error("Error traversing XML tree: {0}")]
    Node(#[from] super::node::NodeError),
    #[error("Encountered unsupported element `{name}` at that location")]
    UnsupportedElement { name: String, position: usize },
    #[error("Encountered unsupported value `{value}` for attribute `{name}` in `{element}`")]
    UnsupportedAttributeValue {
        name: String,
        value: String,
        element: String,
        position: usize,
    },
    #[error("Encountered unexpected attribute `{name}` in `{element}`")]
    UnexpectedAttribute {
        name: String,
        element: String,
        position: usize,
    },
    #[error("Missing element `{name}` in `{parent}`")]
    MissingElement {
        name: String,
        parent: String,
        position: usize,
    },
    // #[error("Missing attribute `{name}` in `{element}`")]
    // MissingAttribute {
    //     name: String,
    //     element: String,
    //     position: usize,
    // },
    #[error("Missing namespace for `{prefix}`")]
    MissingNamespace { prefix: String, position: usize },
    // #[error("Multiple types found inside `{name}`")]
    // MultipleTypes { name: String, position: usize },
    #[error("Failed to parse int value")]
    ParseInt {
        err: std::num::ParseIntError,
        position: usize,
    },
    #[error("Failed to parse decimal value")]
    ParseDecimal {
        err: rust_decimal::Error,
        position: usize,
    },
    #[error("Unsupported XSD type {name}")]
    UnsupportedType { name: String, position: usize },
}

impl XsdError {
    pub fn position(&self) -> Option<usize> {
        match self {
            XsdError::Xml(_) => None,
            // XsdError::Import(_) => None,
            XsdError::Node(err) => err.position(),
            XsdError::UnsupportedElement { position, .. } => Some(*position),
            XsdError::UnsupportedAttributeValue { position, .. } => Some(*position),
            XsdError::UnexpectedAttribute { position, .. } => Some(*position),
            XsdError::MissingElement { position, .. } => Some(*position),
            // XsdError::MissingAttribute { position, .. } => Some(*position),
            XsdError::MissingNamespace { position, .. } => Some(*position),
            // XsdError::MultipleTypes { position, .. } => Some(*position),
            XsdError::ParseInt { position, .. } => Some(*position),
            XsdError::ParseDecimal { position, .. } => Some(*position),
            XsdError::UnsupportedType { position, .. } => Some(*position),
        }
    }
}
