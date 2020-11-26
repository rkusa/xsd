use crate::ast::{ElementDefinition, Name, Root};
use crate::xsd::context::{Context, NS_XSD};
use crate::xsd::node::Node;
use crate::xsd::XsdError;

pub fn parse<'a, 'input>(
    node: Node<'a, 'input>,
    parent: &Name,
    ctx: &mut Context<'a, 'input>,
) -> Result<Root, XsdError>
where
    'a: 'input,
{
    let mut children = node.children().namespace(NS_XSD).collect();
    // TODO: (annotation?,(simpleContent|complexContent|)
    // TODO: annotation
    // TODO: simpleContent xor complexContent xor the following

    if let Some(child) = children.remove("simpleContent", Some(NS_XSD)) {
        return Ok(Root::Element(super::simple_content::parse(child, ctx)?));
    }

    let content = if let Some(child) = children.remove("sequence", Some(NS_XSD)) {
        // TODO: or all, choice
        Some(super::sequence::parse(child, parent, ctx)?)
    } else {
        None
    };

    // read all attributes
    let mut attributes = Vec::new();
    while let Some(child) = children.remove("attribute", Some(NS_XSD)) {
        attributes.push(super::attribute::parse(child, ctx)?);
    }

    children.prevent_unvisited_children()?;

    Ok(Root::Element(ElementDefinition {
        attributes,
        content,
    }))
}
