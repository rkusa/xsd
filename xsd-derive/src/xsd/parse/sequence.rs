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
                ctx.add_root(
                    virtual_name.clone(),
                    Root::Choice(ChoiceDefinition {
                        variants,
                        is_virtual: true,
                    }),
                );

                Leaf {
                    name: leaf_name,
                    definition: LeafDefinition {
                        content: LeafContent::Named(virtual_name),
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
                let mut virtual_name = String::new();
                for v in &leaves {
                    if !v.name.name.is_empty() {
                        virtual_name += (&v.name.name[0..1]).to_ascii_uppercase().as_str();
                        virtual_name += &v.name.name[1..];
                    }
                }

                let leaf_name = ctx.get_node_name(&virtual_name, false);

                let virtual_name = parent.name.to_string() + &virtual_name;
                let virtual_name = ctx.get_node_name(&virtual_name, false);
                ctx.add_root(
                    virtual_name.clone(),
                    Root::Element(ElementDefinition {
                        attributes: Vec::new(),
                        content: Some(ElementContent::Leaves(leaves)),
                        is_virtual: true,
                    }),
                );

                Leaf {
                    name: leaf_name,
                    definition: LeafDefinition {
                        content: LeafContent::Named(virtual_name),
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
