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
pub struct Node<'a>(roxmltree::Node<'a, 'a>);

pub fn decode(input: &str) -> Result<Document<'_>, FromXmlError> {
    let doc = roxmltree::Document::parse(&input)?;
    Ok(Document(doc))
}

impl<'a> Document<'a> {
    pub fn child(&'a self, name: &'a str, namespace: Option<&'a str>) -> Child<'a> {
        let root = self.0.root_element();
        let tag_name = root.tag_name();
        let node =
            if !root.is_element() || tag_name.name() != name || tag_name.namespace() != namespace {
                None
            } else {
                Some(Node(root))
            };
        Child {
            name,
            namespace,
            node,
        }
    }
}

pub struct Child<'a> {
    name: &'a str,
    namespace: Option<&'a str>,
    node: Option<Node<'a>>,
}

impl<'a> Child<'a> {
    pub fn take(self) -> Option<Node<'a>> {
        self.node
    }

    pub fn try_take(mut self) -> Result<Node<'a>, FromXmlError> {
        let node = self.node.take();
        node.ok_or_else(|| FromXmlError::MissingElement {
            name: self.name.to_string(),
            namespace: self.namespace.map(String::from),
        })
    }
}

impl<'a> Node<'a> {
    pub fn child(&'a self, name: &'a str, namespace: Option<&'a str>) -> Child<'a> {
        let node = self
            .0
            .children()
            .find(|child| {
                if !child.is_element() {
                    return false;
                }

                let tag_name = child.tag_name();
                tag_name.name() == name && tag_name.namespace() == namespace
            })
            .map(Node);
        Child {
            name,
            namespace,
            node,
        }
    }

    pub fn try_take(&self) -> Result<&Node<'a>, FromXmlError> {
        Ok(self)
    }

    pub fn children(
        &'a self,
        name: &'a str,
        namespace: Option<&'a str>,
    ) -> impl Iterator<Item = Node<'a>> {
        self.0
            .children()
            .filter(move |child| {
                if !child.is_element() {
                    return false;
                }

                let tag_name = child.tag_name();
                tag_name.name() == name && tag_name.namespace() == namespace
            })
            .map(Node)
    }

    pub fn attribute(&'a self, name: &'a str) -> Option<&'a str> {
        self.0.attribute(name)
    }

    pub fn try_attribute(&'a self, name: &'a str) -> Result<&'a str, FromXmlError> {
        self.attribute(name)
            .ok_or_else(|| FromXmlError::MissingAttribute {
                name: name.to_string(),
            })
    }

    pub fn text(&self) -> Result<&str, FromXmlError> {
        self.0.text().ok_or_else(|| FromXmlError::TextExpected {
            name: self.0.tag_name().name().to_string(),
        })
    }

    pub fn range(&self) -> Range<usize> {
        self.0.range()
    }
}
