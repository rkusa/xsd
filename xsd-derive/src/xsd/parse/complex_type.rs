use crate::ast::{ElementDefinition, Kind, Name};
use crate::xsd::context::{Context, NS_XSD};
use crate::xsd::node::Node;
use crate::xsd::XsdError;

pub fn parse<'a, 'input>(
    node: Node<'a, 'input>,
    parent: &Name,
    ctx: &mut Context<'a, 'input>,
    kind: Kind,
) -> Result<ElementDefinition, XsdError>
where
    'a: 'input,
{
    let mut children = node.children().namespace(NS_XSD).collect();
    // TODO: (annotation?,(simpleContent|complexContent|)
    // TODO: annotation
    // TODO: simpleContent xor complexContent xor the following

    let content = if let Some(child) = children.remove("sequence", Some(NS_XSD)) {
        super::sequence::parse(child, parent, ctx)?
    }
    // TODO: or all, choice
    else {
        return Err(XsdError::MissingElement {
            name: "sequence".to_string(),
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
        kind,
        attributes,
        content,
    })
}
