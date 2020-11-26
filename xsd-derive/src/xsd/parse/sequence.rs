use crate::ast::{ElementContent, Leaf, Name};
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
    node.prevent_unvisited_attributes()?;

    let mut leaves = Vec::new();
    for child in node.children().namespace(NS_XSD).iter() {
        match child.name() {
            "element" => {
                let name = ctx.get_node_name(&child.try_attribute("name")?.value(), false);
                leaves.push(Leaf {
                    name,
                    definition: super::element::parse_child(child, parent, ctx)?,
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
