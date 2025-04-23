use crate::ast::{
    ChoiceDefinition, ElementContent, ElementDefinition, Leaf, LeafContent, LeafDefinition, Name,
    Root,
};
use crate::xsd::context::{Context, NS_XSD};
use crate::xsd::error::XsdError;
use crate::xsd::node::Node;

use super::element::{parse_max_occurs, parse_min_occurs};

pub fn parse<'a, 'input>(
    node: Node<'a, 'input>,
    parent: &Name,
    ctx: &mut Context<'input>,
) -> Result<Root, XsdError>
where
    'a: 'input,
{
    if node.attribute("mixed").map(|a| a.value()).as_deref() == Some("true") {
        return Err(XsdError::UnsupportedAttributeValue {
            name: "mixed".to_string(),
            value: "true".to_string(),
            element: node.name().to_string(),
            range: node.range(),
        });
    }
    node.prevent_unvisited_attributes()?;

    let mut children = node.children().namespace(NS_XSD).collect();
    let docs = super::parse_annotation(children.remove("annotation", Some(NS_XSD)))?;
    // TODO: (annotation?,(simpleContent|complexContent|)
    // TODO: simpleContent xor complexContent xor the following

    if let Some(child) = children.remove("simpleContent", Some(NS_XSD)) {
        children.prevent_unvisited_children()?;
        return Ok(Root::Element(
            super::simple_content::parse(child, parent, ctx)?.with_docs(docs),
        ));
    }
    if let Some(child) = children.remove("complexContent", Some(NS_XSD)) {
        children.prevent_unvisited_children()?;
        return Ok(Root::Element(
            super::complex_content::parse(child, parent, ctx)?.with_docs(docs),
        ));
    }

    let content = if let Some(child) = children
        .remove("sequence", Some(NS_XSD))
        .or_else(|| children.remove("all", Some(NS_XSD)))
    {
        let is_unordered = child.name() == "all";
        let min_occurs = parse_min_occurs(child.attribute("minOccurs"))?;
        let max_occurs = parse_max_occurs(child.attribute("maxOccurs"))?;
        let mut leaves = super::sequence::parse(child, parent, ctx)?;
        if is_unordered {
            for leaf in &mut leaves {
                leaf.is_unordered = true;
            }
        }

        if max_occurs.is_vec() || min_occurs != Default::default() {
            let leaf_name = super::derive_virtual_name(leaves.iter().map(|v| &v.name), ctx, true);
            let root_name = super::derive_virtual_name(vec![parent, &leaf_name], ctx, false);

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

            Some(ElementContent::Leaves(vec![Leaf {
                name: leaf_name,
                definition: LeafDefinition {
                    content: LeafContent::Named(root_name),
                    restrictions: Vec::new(),
                    docs: None,
                },
                is_unordered: false,
                is_virtual: true,
                min_occurs,
                max_occurs,
            }]))
        } else {
            Some(ElementContent::Leaves(leaves))
        }
    } else if let Some(child) = children.remove("choice", Some(NS_XSD)) {
        let variants = super::choice::parse(child, parent, ctx)?;
        let root_name =
            super::derive_virtual_name(vec![parent, &ctx.get_node_name("Data", false)], ctx, false);

        ctx.add_root(
            root_name.clone(),
            Root::Choice(ChoiceDefinition {
                variants,
                is_virtual: false,
                docs: None,
            }),
        );
        ctx.discover_type(&root_name, Some(parent));

        Some(ElementContent::Leaf(
            root_name.clone(),
            LeafDefinition {
                content: LeafContent::Named(root_name),
                restrictions: Vec::new(),
                docs: None,
            },
        ))
    } else {
        None
    };

    // read all attributes
    let mut attributes = Vec::new();
    while let Some(child) = children.remove("attribute", Some(NS_XSD)) {
        if let Some(attr) = super::attribute::parse(child, parent, ctx)? {
            attributes.push(attr);
        }
    }

    children.prevent_unvisited_children()?;

    Ok(Root::Element(ElementDefinition {
        attributes,
        content,
        is_virtual: false,
        docs,
    }))
}
