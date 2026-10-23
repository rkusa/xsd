use std::cell::{OnceCell, RefCell};
use std::collections::VecDeque;
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
    #[error("Could not find valid variant for choice `{name}`")]
    MissingVariant { name: String },
    #[error("Failed to parse type {type_} from {value}: {err}")]
    ParseType {
        type_: String,
        value: String,
        err: Box<dyn std::error::Error + Sync + Send>,
    },
    #[error("Expected fixed value `{expected}`, but received `{received}`")]
    FixedMismatch {
        expected: &'static str,
        received: String,
    },
    #[error("infallible")]
    Infallible(#[from] std::convert::Infallible),
}

pub struct Document<'a>(roxmltree::Document<'a>);
pub struct Node<'a> {
    node: roxmltree::Node<'a, 'a>,
    children: RefCell<Children<'a>>,
    /// Text content that is split into multiple text nodes (e.g. by comments) and thus had to
    /// be joined.
    joined_text: OnceCell<String>,
}

struct Children<'a> {
    children: Box<dyn Iterator<Item = roxmltree::Node<'a, 'a>> + 'a>,
    peeked: VecDeque<roxmltree::Node<'a, 'a>>,
    next: Option<roxmltree::Node<'a, 'a>>,
}

pub fn decode(input: &str) -> Result<Document<'_>, FromXmlError> {
    let doc = roxmltree::Document::parse(input)?;
    Ok(Document(doc))
}

impl Document<'_> {
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
                peeked: VecDeque::new(),
                next: None,
            }),
            joined_text: OnceCell::new(),
            node,
        }
    }

    pub fn peek_child(&self, name: &str, namespace: Option<&str>) -> bool {
        let mut children = self.children.borrow_mut();
        if children.next.is_none() {
            children.next = children
                .peeked
                .pop_front()
                .or_else(|| children.children.next());
        }
        if let Some(next) = &children.next {
            let tag_name = next.tag_name();
            return tag_name.name() == name && tag_name.namespace() == namespace;
        }
        false
    }

    pub fn next_child(&self, name: &str, namespace: Option<&str>) -> Option<Node<'_>> {
        let mut children = self.children.borrow_mut();
        if let Some(next) = children
            .next
            .take()
            .or_else(|| children.peeked.pop_front())
            .or_else(|| children.children.next())
        {
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

    pub fn child(&self, name: &str, namespace: Option<&str>) -> Option<Node<'_>> {
        let mut children = self.children.borrow_mut();
        if let Some(next) = children.next.take() {
            let tag_name = next.tag_name();
            if tag_name.name() == name && tag_name.namespace() == namespace {
                return Some(Node::new(next));
            }
            children.next = Some(next)
        }

        for (ix, child) in children.peeked.iter().enumerate() {
            let tag_name = child.tag_name();
            if tag_name.name() == name && tag_name.namespace() == namespace {
                return children.peeked.remove(ix).map(Node::new);
            }
        }

        #[allow(clippy::while_let_on_iterator)]
        while let Some(child) = children.children.next() {
            let tag_name = child.tag_name();
            if tag_name.name() == name && tag_name.namespace() == namespace {
                return Some(Node::new(child));
            }
            children.peeked.push_back(child);
        }

        None
    }

    pub fn try_child(&self, name: &str, namespace: Option<&str>) -> Result<Node<'_>, FromXmlError> {
        self.child(name, namespace)
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

    /// The element's text content. Comments, processing instructions and child elements (e.g.
    /// inside of `xs:anyType` elements) are ignored.
    pub fn text(&self) -> Result<&str, FromXmlError> {
        let mut text: Option<&'a str> = None;
        let mut is_split = false;
        for child in self.node.children() {
            if child.is_text() {
                if text.is_some() {
                    is_split = true;
                } else {
                    text = child.text();
                }
            }
        }

        if !is_split {
            return Ok(text.unwrap_or(""));
        }

        Ok(self.joined_text.get_or_init(|| {
            self.node
                .children()
                .filter_map(|child| if child.is_text() { child.text() } else { None })
                .collect()
        }))
    }

    pub fn range(&self) -> Range<usize> {
        self.node.range()
    }
}
