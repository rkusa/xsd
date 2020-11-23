use std::cell::Cell;

use super::context::Context;
use super::error::XsdError;
use super::node::Node;
use crate::types::{ElementContent, ElementDefinition};

pub struct Lazy<'a, 'input> {
    node: Node<'a, 'input>,
    content: Cell<Option<ElementDefinition>>,
    requested: Cell<bool>,
}

impl<'a, 'input> Lazy<'a, 'input> {
    pub fn new(node: Node<'a, 'input>) -> Self {
        Lazy {
            node,
            content: Cell::new(None),
            requested: Cell::new(false),
        }
    }

    pub fn try_get(&self, ctx: &Context<'_, '_>) -> Result<ElementDefinition, XsdError> {
        let mut content = self.content.take();
        if content.is_none() && self.requested.get() {
            return Err(XsdError::CircularType {
                name: self.node.name().to_string(),
                range: self.node.range(),
            });
        }
        self.requested.set(true);
        self.content.set(content.clone());

        match content.take() {
            Some(def) => Ok(def),
            None => {
                let def = self.parse(ctx)?;
                self.content.set(Some(def.clone()));
                Ok(def)
            }
        }
    }

    fn parse(&self, ctx: &Context<'_, '_>) -> Result<ElementDefinition, XsdError> {
        match self.node.name() {
            "element" => super::parse::element::parse(&self.node, ctx),
            child_name => Err(XsdError::UnsupportedElement {
                name: self.node.name().to_string(),
                parent: "schema".to_string(),
                range: self.node.range(),
            }),
        }
    }
}
