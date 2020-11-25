use crate::ast::{ElementContent, ElementDefinition, Kind, Leaf, LeafContent, Name};
use crate::xsd::context::{Context, NS_XSD};
use crate::xsd::node::Node;
use crate::xsd::XsdError;

pub fn parse<'a, 'input>(
    node: Node<'a, 'input>,
    parent: &Name,
    ctx: &mut Context<'a, 'input>,
) -> Result<ElementContent, XsdError>
where
    'a: 'input,
{
    let mut leaves = Vec::new();

    for child in node.children().namespace(NS_XSD).iter() {
        match child.name() {
            "element" => {
                let name = ctx.get_node_name(&child.try_attribute("name")?.value(), false);
                // TODO: impl attributes for leaves
                let ElementDefinition {
                    kind: _,
                    attributes: _,
                    content,
                } = super::element::parse(child, parent, ctx, Kind::Child)?;
                match content {
                    ElementContent::Literal(literal) => {
                        leaves.push(Leaf {
                            name,
                            content: LeafContent::Literal(literal),
                        });
                    }
                    ElementContent::Reference(ref_name) => leaves.push(Leaf {
                        name,
                        content: LeafContent::Named(ref_name),
                    }),
                    ElementContent::Leaves(_leaves) => {
                        unimplemented!("sequence ElementContent::Leaves")
                    }
                }
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
