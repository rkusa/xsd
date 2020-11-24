use crate::types::{Element, ElementContent, Name, Namespace};
use crate::xsd::context::{Context, NS_XSD};
use crate::xsd::node::Node;
use crate::xsd::XsdError;

pub fn parse(node: &Node<'_, '_>, ctx: &Context<'_, '_>) -> Result<ElementContent, XsdError> {
    let mut inner = Vec::new();

    for child in node.children().namespace(NS_XSD).iter() {
        match child.name() {
            "element" => {
                inner.push(Element {
                    name: Name::new(child.try_attribute("name")?.value(), Namespace::Target),
                    definition: super::element::parse(&child, ctx)?,
                });
            }
            child_name => {
                return Err(XsdError::UnsupportedElement {
                    name: child_name.to_string(),
                    parent: node.name().to_string(),
                    range: child.range(),
                })
            }
        }
    }

    Ok(ElementContent::Elements(inner))
}
