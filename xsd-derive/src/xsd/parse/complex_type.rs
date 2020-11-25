use crate::ast::{ElementContent, Name};
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
    let mut children = node.children().namespace(NS_XSD).collect();
    let content = if let Some(child) = children.remove("sequence", Some(NS_XSD)) {
        super::sequence::parse(child, parent, ctx)?
    } else {
        return Err(XsdError::MissingElement {
            name: "sequence".to_string(),
            parent: node.name().to_string(),
            range: node.range(),
        });
    };

    children.prevent_unvisited_children()?;

    Ok(content)
}
