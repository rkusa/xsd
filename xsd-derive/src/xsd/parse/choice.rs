use crate::ast::{
    ElementContent, ElementDefinition, Leaf, LeafContent, LeafDefinition, MaxOccurs, MinOccurs,
    Name, Root,
};
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
        let variant = match child.name() {
            "element" => super::element::parse_child(child, parent, ctx)?,
            "sequence" => {
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
                    }),
                );

                Leaf {
                    name: leaf_name,
                    definition: LeafDefinition {
                        content: LeafContent::Named(root_name),
                        restrictions: Vec::new(),
                    },
                    is_virtual: true,
                    min_occurs: MinOccurs::default(),
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
        variants.push(variant);
    }

    Ok(variants)
}
