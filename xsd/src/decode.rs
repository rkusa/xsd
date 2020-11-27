use std::cell::RefCell;
use std::ops::Range;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum FromXmlError {
    #[error("XML error: {0}")]
    Xml(#[from] roxmltree::Error),
    #[error("Missing required element {name} ({namespace:?})")]
    MissingElement {
        name: String,
        namespace: Option<String>,
    },
    #[error("Missing required attribute {name}")]
    MissingAttribute { name: String },
    #[error("Expected element {name} to contain text content")]
    TextExpected { name: String },
    #[error("Encountered invalid enum variant {name}")]
    InvalidVariant { name: String },
    #[error("Could not find valid variant for choice")]
    MissingVariant,
    #[error("Failed to parse type {type_} from {value}: {err}")]
    ParseType {
        type_: String,
        value: String,
        err: Box<dyn std::error::Error + Sync + Send>,
    },
}

pub struct Document<'a>(roxmltree::Document<'a>);
pub struct Node<'a> {
    node: roxmltree::Node<'a, 'a>,
    children: RefCell<Children<'a>>,
}

struct Children<'a> {
    children: Box<dyn Iterator<Item = roxmltree::Node<'a, 'a>> + 'a>,
    next: Option<roxmltree::Node<'a, 'a>>,
}

pub fn decode(input: &str) -> Result<Document<'_>, FromXmlError> {
    let doc = roxmltree::Document::parse(&input)?;
    Ok(Document(doc))
}

impl<'a> Document<'a> {
    pub fn child(&self, name: &str, namespace: Option<&str>) -> Option<Node<'_>> {
        let root = self.0.root_element();
        let tag_name = root.tag_name();
        if !root.is_element() || tag_name.name() != name || tag_name.namespace() != namespace {
            None
        } else {
            Some(Node::new(root))
        }
    }

    pub fn try_child(&self, name: &str, namespace: Option<&str>) -> Result<Node<'_>, FromXmlError> {
        self.child(name, namespace)
            .ok_or_else(|| FromXmlError::MissingElement {
                name: name.to_string(),
                namespace: namespace.map(String::from),
            })
    }
}

impl<'a> Node<'a> {
    fn new(node: roxmltree::Node<'a, 'a>) -> Self {
        Node {
            children: RefCell::new(Children {
                children: Box::new(node.children().filter(|c| c.is_element())),
                next: None,
            }),
            node,
        }
    }

    pub fn next_child(&self, name: &str, namespace: Option<&str>) -> Option<Node<'_>> {
        let mut children = self.children.borrow_mut();
        if let Some(next) = children.next.take().or_else(|| children.children.next()) {
            let tag_name = next.tag_name();
            if tag_name.name() == name && tag_name.namespace() == namespace {
                return Some(Node::new(next));
            }
            children.next = Some(next)
        }
        None
    }

    pub fn try_next_child(
        &self,
        name: &str,
        namespace: Option<&str>,
    ) -> Result<Node<'_>, FromXmlError> {
        self.next_child(name, namespace)
            .ok_or_else(|| FromXmlError::MissingElement {
                name: name.to_string(),
                namespace: namespace.map(String::from),
            })
    }

    pub fn attribute(&self, name: &str) -> Option<&'a str> {
        self.node.attribute(name)
    }

    pub fn try_attribute(&self, name: &str) -> Result<&'a str, FromXmlError> {
        self.attribute(name)
            .ok_or_else(|| FromXmlError::MissingAttribute {
                name: name.to_string(),
            })
    }

    pub fn text(&self) -> Result<&str, FromXmlError> {
        self.node.text().ok_or_else(|| FromXmlError::TextExpected {
            name: self.node.tag_name().name().to_string(),
        })
    }

    pub fn range(&self) -> Range<usize> {
        self.node.range()
    }
}
