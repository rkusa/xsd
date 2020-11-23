use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum GeneratorError {
    #[error("Failed to load WSDL file {file}: {err}")]
    Open {
        #[source]
        err: io::Error,
        file: String,
    },
    // #[error("XML parsing error")]
    // Xml(#[from] roxmltree::Error),
    #[error("Error parsing XML Schema: {0}")]
    Xsd(#[from] crate::xsd::Error),
    // #[error("Error traversing XML tree: {0}")]
    // Node(#[from] xsd::node::NodeError),
    // #[error("Unsupported binding transport {0} (only HTTP is supported)")]
    // UnsupportedTransport(String),
    // #[error("Unsupported binding style {0} (only document is supported)")]
    // UnsupportedStyle(String),
    // #[error("Unsupported attribute {name} on element {element}")]
    // UnsupportedAttribute { name: String, element: String },
    // #[error("Unsupported attribute {name} value {value} on element {element}")]
    // UnsupportedAttributeValue {
    //     name: String,
    //     value: String,
    //     element: String,
    // },
    // #[error("Multiple parts in a wsdl:message are currently unsupported")]
    // UnsupportedMultipleParts,
    // #[error("Could not find types (schema) for namespace {namespace}")]
    // MissingSchema { namespace: String },
    // #[error("Could not find type {name} in schema for prefix {prefix}")]
    // MissingType { name: String, prefix: String },
}
