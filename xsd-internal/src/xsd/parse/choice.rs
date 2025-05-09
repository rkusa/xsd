use crate::ast::{
    ElementContent, ElementDefinition, Leaf, LeafContent, LeafDefinition, Name, Root,
};
use crate::xsd::context::{Context, NS_XSD};
use crate::xsd::error::XsdError;
use crate::xsd::node::Node;

use super::element::{parse_max_occurs, parse_min_occurs};

pub fn parse<'a, 'input>(
    node: Node<'a, 'input>,
    parent: &Name,
    ctx: &mut Context<'input>,
) -> Result<Vec<Leaf>, XsdError>
where
    'a: 'input,
{
    node.prevent_unvisited_attributes()?;

    let mut variants = Vec::new();
    for child in node.children().namespace(NS_XSD).iter() {
        let variant = match child.name() {
            "element" => super::element::parse_child(child, parent, ctx)?,
            "sequence" => {
                let docs = super::parse_annotation(child.child("annotation", Some(NS_XSD)))?;
                let min_occurs = parse_min_occurs(child.attribute("minOccurs"))?;
                let max_occurs = parse_max_occurs(child.attribute("maxOccurs"))?;

                let leaves = super::sequence::parse(child, parent, ctx)?;
                let leaf_name =
                    super::derive_virtual_name(leaves.iter().map(|v| &v.name), ctx, true);
                let root_name = super::derive_virtual_name(
                    vec![parent, &leaf_name, &ctx.get_node_name("Variant", false)],
                    ctx,
                    false,
                );

                ctx.add_root(
                    root_name.clone(),
                    Root::Element(ElementDefinition {
                        attributes: Vec::new(),
                        content: Some(ElementContent::Leaves(leaves)),
                        is_virtual: true,
                        docs: None,
                    }),
                );
                ctx.discover_type(&root_name, Some(parent));

                Leaf {
                    name: leaf_name,
                    definition: LeafDefinition {
                        content: LeafContent::Named(root_name),
                        restrictions: Vec::new(),
                        docs,
                    },
                    is_unordered: false,
                    is_virtual: true,
                    min_occurs,
                    max_occurs,
                }
            }
            "annotation" => continue,
            child_name => {
                return Err(XsdError::UnsupportedElement {
                    name: child_name.to_string(),
                    range: child.range(),
                })
            }
        };
        variants.push(variant);
    }

    Ok(variants)
}
