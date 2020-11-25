use std::cell::Cell;
use std::ops::Range;

use super::context::Context;
use super::error::XsdError;
use super::node::Node;
use crate::ast::{ElementDefinition, Kind, Name, Namespace};

pub struct Lazy<'a, 'input> {
    name: String,
    range: Range<usize>,
    kind: Kind,
    content: Cell<Content<'a, 'input>>,
}

enum Content<'a, 'input> {
    Requested,
    Node(Node<'a, 'input>),
    Parsed(ElementDefinition),
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
    pub fn new(node: Node<'a, 'input>, kind: Kind) -> Self {
        Lazy {
            name: node.name().to_string(),
            range: node.range(),
            kind,
            content: Cell::new(Content::Node(node)),
        }
    }

    pub fn try_get(&self, ctx: &mut Context<'a, 'input>) -> Result<ElementDefinition, XsdError> {
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
    ) -> Result<ElementDefinition, XsdError> {
        let parent = Name::new("", Namespace::None);
        match node.name() {
            "element" => super::parse::element::parse(node, &parent, ctx, self.kind),
            "complexType" => Ok(ElementDefinition {
                kind: Kind::Virtual,
                content: super::parse::complex_type::parse(node, &parent, ctx)?,
            }),
            child_name => Err(XsdError::UnsupportedElement {
                name: child_name.to_string(),
                parent: "schema".to_string(),
                range: node.range(),
            }),
        }
    }
}
