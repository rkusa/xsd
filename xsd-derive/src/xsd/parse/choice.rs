use crate::ast::{Leaf, Name};
use crate::xsd::context::{Context, NS_XSD};
use crate::xsd::node::Node;
use crate::xsd::XsdError;

pub fn parse<'a, 'input>(
    node: Node<'a, 'input>,
    parent: &Name,
    ctx: &mut Context<'a, 'input>,
) -> Result<Vec<Leaf>, XsdError>
where
    'a: 'input,
{
    node.prevent_unvisited_attributes()?;

    let mut variants = Vec::new();
    for child in node.children().namespace(NS_XSD).iter() {
        match child.name() {
            "element" => {
                let name = ctx.get_node_name(&child.try_attribute("name")?.value(), false);
                variants.push(Leaf {
                    name,
                    definition: super::element::parse_child(child, parent, ctx)?,
                    is_virtual: false,
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

    Ok(variants)
}
