use crate::types::{ElementContent, Leaf, LeafContent};
use crate::xsd::context::{Context, NS_XSD};
use crate::xsd::node::Node;
use crate::xsd::XsdError;

pub fn parse(node: &Node<'_, '_>, ctx: &Context<'_, '_>) -> Result<ElementContent, XsdError> {
    let mut leaves = Vec::new();

    for child in node.children().namespace(NS_XSD).iter() {
        match child.name() {
            "element" => {
                leaves.push(Leaf {
                    name: ctx.get_node_name(child.try_attribute("name")?.value(), false),
                    content: match super::element::parse(&child, ctx)?.content {
                        ElementContent::Literal(literal) => LeafContent::Literal(literal),
                        ElementContent::Leaves(_) => unimplemented!("nested elements"),
                    },
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

    Ok(ElementContent::Leaves(leaves))
}
