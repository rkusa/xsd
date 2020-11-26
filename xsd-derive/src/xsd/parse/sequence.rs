use crate::ast::{ElementContent, Leaf, LeafContent, LeafDefinition, Name, Root};
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
                    is_virtual: false,
                });
            }
            "choice" => {
                let variants = super::choice::parse(child, parent, ctx)?;
                let mut virtual_name = String::new();
                for v in &variants {
                    if !v.name.name.is_empty() {
                        virtual_name += (&v.name.name[0..1]).to_ascii_uppercase().as_str();
                        virtual_name += &v.name.name[1..];
                    }
                }

                let leaf_name = ctx.get_node_name(&virtual_name, false);

                let virtual_name = parent.name.to_string() + &virtual_name;
                let virtual_name = ctx.get_node_name(&virtual_name, false);
                ctx.add_root(virtual_name.clone(), Root::Choice(variants));

                leaves.push(Leaf {
                    name: leaf_name,
                    definition: LeafDefinition {
                        content: LeafContent::Named(virtual_name),
                        restrictions: Vec::new(),
                    },
                    is_virtual: true,
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
