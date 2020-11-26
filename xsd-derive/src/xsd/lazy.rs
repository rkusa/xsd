use std::cell::Cell;
use std::ops::Range;

use super::context::Context;
use super::error::XsdError;
use super::node::Node;
use crate::ast::{Name, Namespace, Root};

pub struct Lazy<'a, 'input> {
    name: String,
    range: Range<usize>,
    content: Cell<Content<'a, 'input>>,
}

enum Content<'a, 'input> {
    Requested,
    Node(Node<'a, 'input>),
    Parsed(Root),
}

impl<'a, 'input> Default for Content<'a, 'input> {
    fn default() -> Self {
        Content::Requested
    }
}

impl<'a, 'input> Lazy<'a, 'input>
where
    'a: 'input,
{
    pub fn new(node: Node<'a, 'input>) -> Self {
        Lazy {
            name: node.name().to_string(),
            range: node.range(),
            content: Cell::new(Content::Node(node)),
        }
    }

    pub fn try_get(&self, ctx: &mut Context<'a, 'input>) -> Result<Root, XsdError> {
        let content = self.content.take();
        match content {
            Content::Requested => Err(XsdError::CircularType {
                name: self.name.clone(),
                range: self.range.clone(),
            }),
            Content::Node(node) => {
                let def = self.parse(node, ctx)?;
                self.content.set(Content::Parsed(def.clone()));
                Ok(def)
            }
            Content::Parsed(def) => {
                self.content.set(Content::Parsed(def.clone()));
                Ok(def)
            }
        }
    }

    fn parse(
        &self,
        node: Node<'a, 'input>,
        ctx: &mut Context<'a, 'input>,
    ) -> Result<Root, XsdError> {
        match node.name() {
            "element" => super::parse::element::parse_root(node, ctx),
            "complexType" => {
                super::parse::complex_type::parse(node, &Name::new("", Namespace::None), ctx)
            }
            "simpleType" => super::parse::simple_type::parse(node, ctx),
            child_name => Err(XsdError::UnsupportedElement {
                name: child_name.to_string(),
                parent: "schema".to_string(),
                range: node.range(),
            }),
        }
    }
}
