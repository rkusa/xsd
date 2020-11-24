// TODO: remove
#![allow(unused)]

use std::cell::Cell;
use std::collections::HashMap;
use std::ops::Range;
use std::{borrow::Cow, ops::Deref};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum NodeError {
    #[error("Missing required element {name} ({namespace:?})")]
    MissingElement {
        name: String,
        namespace: Option<String>,
        range: Range<usize>,
    },
    #[error("Cannot find element for filter: {element_name:?}, {element_namespace:?}, {attribute_name:?}, {attribute_name:?}")]
    ElementNotFound {
        element_name: Option<String>,
        element_namespace: Option<String>,
        attribute_name: Option<String>,
        attribute_value: Option<String>,
        range: Range<usize>,
    },
    #[error("Missing required attribute {name}")]
    MissingAttribute { name: String, range: Range<usize> },
    #[error("Expected element {name} to contain text content")]
    TextExpected { name: String, range: Range<usize> },
    #[error("Encountered unsupported attribute `{name}` in `{element}`")]
    UnsupportedAttribute {
        name: String,
        element: String,
        range: Range<usize>,
    },
    #[error("Could not find namespace for URI {uri}")]
    MissingNamespace { uri: String, range: Range<usize> },
}

#[derive(Clone)]
pub struct Node<'a, 'input> {
    name: &'input str,
    inner: roxmltree::Node<'a, 'input>,
    attributes: HashMap<&'a str, Attribute<'a, 'input>>,
}

#[derive(Clone)]
pub struct Attribute<'a, 'input> {
    inner: &'a roxmltree::Attribute<'input>,
    visited: Cell<bool>,
}

impl<'a, 'input> From<roxmltree::Node<'a, 'input>> for Node<'a, 'input> {
    fn from(node: roxmltree::Node<'a, 'input>) -> Self {
        // TODO: throw on non XSD namespace children/attributes instead of ignoring them?
        let attributes = node
            .attributes()
            .iter()
            .map(|attr| {
                // let ns = attr.namespace();
                // TODO: filter by XSD attributes?
                (
                    attr.name(),
                    Attribute {
                        inner: attr,
                        visited: Cell::new(false),
                    },
                )
            })
            .collect();
        Node {
            name: node.tag_name().name(),
            inner: node,
            attributes,
        }
    }
}

impl<'a, 'input> Node<'a, 'input> {
    pub fn name(&'a self) -> &'input str {
        let tag_name = self.inner.tag_name();
        tag_name.name()
    }

    pub fn namespace(&self) -> Option<Cow<'input, str>> {
        // TODO: this currently always returns a &str, but is internally a Cow, find a way to
        // retrieve the Cow instead of the &str to avoid unnecessary allocations
        self.inner
            .tag_name()
            .namespace()
            .map(|v| Cow::Owned(v.to_string()))
    }

    pub fn range(&self) -> Range<usize> {
        self.inner.range()
    }

    pub fn child(&self, name: &str, namespace: Option<&str>) -> Option<Node<'a, 'input>> {
        self.inner
            .children()
            .find(|child| {
                if !child.is_element() {
                    return false;
                }

                let tag_name = child.tag_name();
                tag_name.name() == name && tag_name.namespace() == namespace
            })
            .map(Node::from)
    }

    pub fn try_child(
        &self,
        name: &str,
        namespace: Option<&str>,
    ) -> Result<Node<'a, 'input>, NodeError> {
        self.child(name, namespace)
            .ok_or_else(|| NodeError::MissingElement {
                name: name.to_string(),
                namespace: namespace.map(String::from),
                range: self.range(),
            })
    }

    // TODO: builder pattern instead of search arguments?
    pub fn children<'b>(&'b self) -> ChildrenFilterBuilder<'a, 'input, 'b> {
        ChildrenFilterBuilder {
            node: self,
            filter: ChildrenFilter::default(),
        }
    }

    pub fn attribute<'b>(&'b self, name: &str) -> Option<&'b Attribute<'a, 'input>> {
        self.attributes.get(name).map(|attr| {
            attr.visited.replace(true);
            attr
        })
    }

    pub fn try_attribute<'b>(&'b self, name: &str) -> Result<&'b Attribute<'a, 'input>, NodeError> {
        self.attribute(name)
            .ok_or_else(|| NodeError::MissingAttribute {
                name: name.to_string(),
                range: self.range(),
            })
    }

    pub fn text(&self) -> Option<Cow<'input, str>> {
        self.inner
            .first_child()
            .and_then(|child| if child.is_text() { Some(child) } else { None })
            .and_then(|child| child.text())
            // TODO: this currently always returns a &str, but is internally a Cow, find a way to
            // retrieve the Cow instead of the &str to avoid unnecessary allocations
            .map(|v| Cow::Owned(v.to_string()))
    }

    pub fn try_text(&self) -> Result<Cow<'input, str>, NodeError> {
        self.text().ok_or_else(|| NodeError::TextExpected {
            name: self.name().to_string(),
            range: self.range(),
        })
    }

    pub fn prevent_unvisited_attributes(&self) -> Result<(), NodeError> {
        for (name, attr) in self.attributes.iter() {
            if !attr.visited.get() {
                return Err(NodeError::UnsupportedAttribute {
                    name: name.to_string(),
                    element: self.name().to_string(),
                    range: attr.range(),
                });
            }
        }

        Ok(())
    }

    pub fn try_namespace_prefix(&self, uri: &str) -> Result<Cow<'input, str>, NodeError> {
        match self.inner.lookup_prefix(uri) {
            Some(uri) => {
                // TODO: this currently always returns a &str, but is internally a Cow, find a way to
                // retrieve the Cow instead of the &str to avoid unnecessary allocations
                Ok(Cow::Owned(uri.to_string()))
            }
            None => Err(NodeError::MissingNamespace {
                uri: uri.to_string(),
                range: self.inner.range(),
            }),
        }
    }

    pub fn try_namespace_uri(&self, prefix: &str) -> Result<Cow<'input, str>, NodeError> {
        match self.inner.lookup_namespace_uri(Some(prefix)) {
            Some(uri) => {
                // TODO: this currently always returns a &str, but is internally a Cow, find a way to
                // retrieve the Cow instead of the &str to avoid unnecessary allocations
                Ok(Cow::Owned(uri.to_string()))
            }
            None => Err(NodeError::MissingNamespace {
                uri: prefix.to_string(),
                range: self.inner.range(),
            }),
        }
    }

    pub fn default_namespace(&self) -> Option<&str> {
        self.inner.default_namespace()
    }
}

impl<'a, 'input> Attribute<'a, 'input> {
    pub fn value(&self) -> Cow<'input, str> {
        // TODO: this currently always returns a &str, but is internally a Cow, find a way to
        // retrieve the Cow instead of the &str to avoid unnecessary allocations
        Cow::Owned(self.inner.value().to_string())
    }

    pub fn range(&self) -> Range<usize> {
        self.inner.range()
    }
}

impl NodeError {
    pub fn range(&self) -> Option<&Range<usize>> {
        use NodeError::*;
        match self {
            MissingElement { range, .. } => Some(range),
            ElementNotFound { range, .. } => Some(range),
            MissingAttribute { range, .. } => Some(range),
            MissingNamespace { range, .. } => Some(range),
            TextExpected { range, .. } => Some(range),
            UnsupportedAttribute { range, .. } => Some(range),
        }
    }
}

#[derive(Debug, Default, Clone)]
struct ChildrenFilter<'a> {
    element_name: Option<&'a str>,
    element_namespace: Option<&'a str>,
    // TODO: allow multiple attribute filter
    attribute_name: Option<&'a str>,
    attribute_value: Option<&'a str>,
}

pub struct ChildrenFilterBuilder<'a, 'input, 'b> {
    node: &'b Node<'a, 'input>,
    filter: ChildrenFilter<'a>,
}

impl<'a, 'input, 'b> ChildrenFilterBuilder<'a, 'input, 'b> {
    pub fn element(mut self, name: &'a str) -> Self {
        self.filter.element_name = Some(name);
        self
    }

    pub fn namespace(mut self, namespace: &'a str) -> Self {
        self.filter.element_namespace = Some(namespace);
        self
    }

    pub fn attribute(mut self, name: &'a str, value: &'a str) -> Self {
        self.filter.attribute_name = Some(name);
        self.filter.attribute_value = Some(value);
        self
    }

    pub fn iter(self) -> impl Iterator<Item = Node<'a, 'input>> + 'b {
        self.node
            .inner
            .children()
            .filter(move |child| {
                if !child.is_element() {
                    return false;
                }

                let tag_name = child.tag_name();

                if let Some(name) = self.filter.element_name {
                    if name != tag_name.name() {
                        return false;
                    }
                }

                if let Some(namespace) = self.filter.element_namespace {
                    if Some(namespace) != tag_name.namespace() {
                        return false;
                    }
                }

                if let (Some(name), Some(value)) =
                    (self.filter.attribute_name, self.filter.attribute_value)
                {
                    return child.attribute(name).as_deref() == Some(value);
                }

                true
            })
            .map(Node::from)
    }

    pub fn try_find(self) -> Result<Node<'a, 'input>, NodeError> {
        let filter = self.filter.clone();
        let range = self.node.range();
        self.iter()
            .next()
            .ok_or_else(|| NodeError::ElementNotFound {
                element_name: filter.element_name.map(String::from),
                element_namespace: filter.element_namespace.map(String::from),
                attribute_name: filter.attribute_name.map(String::from),
                attribute_value: filter.attribute_value.map(String::from),
                range,
            })
    }
}

impl<'a, 'input> Deref for Node<'a, 'input> {
    type Target = roxmltree::Node<'a, 'input>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
