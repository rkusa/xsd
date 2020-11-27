use crate::ast::{
    ChoiceDefinition, ElementContent, ElementDefinition, Leaf, LeafContent, LeafDefinition,
    MaxOccurs, Name, Root,
};
use crate::xsd::context::{Context, NS_XSD};
use crate::xsd::node::Node;
use crate::xsd::XsdError;

use super::element::parse_min_occurs;

pub fn parse<'a, 'input>(
    node: Node<'a, 'input>,
    parent: &Name,
    ctx: &mut Context<'a, 'input>,
) -> Result<Vec<Leaf>, XsdError>
where
    'a: 'input,
{
    node.prevent_unvisited_attributes()?;

    let mut leaves = Vec::new();
    for child in node.children().namespace(NS_XSD).iter() {
        let leaf = match child.name() {
            "element" => super::element::parse_child(child, parent, ctx)?,
            "choice" => {
                let min_occurs = parse_min_occurs(child.attribute("minOccurs"))?;

                let variants = super::choice::parse(child, parent, ctx)?;
                let leaf_name = super::derive_virtual_name(variants.iter().map(|v| &v.name), ctx);
                let root_name = super::derive_virtual_name(vec![parent, &leaf_name], ctx);

                ctx.add_root(
                    root_name.clone(),
                    Root::Choice(ChoiceDefinition {
                        variants,
                        is_virtual: true,
                    }),
                );

                Leaf {
                    name: leaf_name,
                    definition: LeafDefinition {
                        content: LeafContent::Named(root_name),
                        restrictions: Vec::new(),
                    },
                    is_virtual: true,
                    min_occurs,
                    max_occurs: MaxOccurs::default(),
                }
            }
            "sequence" => {
                let min_occurs = parse_min_occurs(child.attribute("minOccurs"))?;

                let leaves = parse(child, parent, ctx)?;
                let leaf_name = super::derive_virtual_name(leaves.iter().map(|v| &v.name), ctx);
                let root_name = super::derive_virtual_name(vec![parent, &leaf_name], ctx);

                ctx.add_root(
                    root_name.clone(),
                    Root::Element(ElementDefinition {
                        attributes: Vec::new(),
                        content: Some(ElementContent::Leaves(leaves)),
                        is_virtual: true,
                    }),
                );

                Leaf {
                    name: leaf_name,
                    definition: LeafDefinition {
                        content: LeafContent::Named(root_name),
                        restrictions: Vec::new(),
                    },
                    is_virtual: true,
                    min_occurs,
                    max_occurs: MaxOccurs::default(),
                }
            }
            child_name => {
                return Err(XsdError::UnsupportedElement {
                    name: child_name.to_string(),
                    parent: node.name().to_string(),
                    range: child.range(),
                })
            }
        };
        leaves.push(leaf);
    }

    Ok(leaves)
}
