use crate::ast::{ElementDefinition, Name};
use crate::xsd::context::{Context, NS_XSD};
use crate::xsd::node::Node;
use crate::xsd::XsdError;

pub fn parse<'a, 'input>(
    node: Node<'a, 'input>,
    parent: &Name,
    ctx: &mut Context<'a, 'input>,
) -> Result<ElementDefinition, XsdError>
where
    'a: 'input,
{
    let mut children = node.children().namespace(NS_XSD).collect();
    // TODO: (annotation?,(simpleContent|complexContent|)
    // TODO: annotation
    // TODO: simpleContent xor complexContent xor the following

    if let Some(child) = children.remove("simpleContent", Some(NS_XSD)) {
        return super::simple_content::parse(child, ctx);
    }

    let content = if let Some(child) = children.remove("sequence", Some(NS_XSD)) {
        // TODO: or all, choice
        super::sequence::parse(child, parent, ctx)?
    } else {
        return Err(XsdError::MissingElement {
            name: "sequence|simpleContent".to_string(),
            parent: node.name().to_string(),
            range: node.range(),
        });
    };

    // read all attributes
    let mut attributes = Vec::new();
    while let Some(child) = children.remove("attribute", Some(NS_XSD)) {
        attributes.push(super::attribute::parse(child, ctx)?);
    }

    children.prevent_unvisited_children()?;

    Ok(ElementDefinition {
        attributes,
        content,
    })
}
