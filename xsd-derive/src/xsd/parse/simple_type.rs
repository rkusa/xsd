use crate::ast::{ElementContent, LeafContent};
use crate::xsd::context::{Context, NS_XSD};
use crate::xsd::node::Node;
use crate::xsd::XsdError;

pub fn parse<'a, 'input>(
    node: Node<'a, 'input>,
    ctx: &mut Context<'a, 'input>,
) -> Result<ElementContent, XsdError>
where
    'a: 'input,
{
    let mut children = node.children().namespace(NS_XSD).collect();
    let content = if let Some(child) = children.remove("restriction", Some(NS_XSD)) {
        let attr = child.try_attribute("base")?;
        let type_name = ctx.get_type_name(&attr)?;
        match type_name {
            LeafContent::Literal(literal) => ElementContent::Literal(literal),
            LeafContent::Named(_) => {
                return Err(XsdError::UnsupportedAttributeValue {
                    name: "base".to_string(),
                    value: attr.value().to_string(),
                    element: "restriction".to_string(),
                    range: attr.range(),
                });
            }
        }
    } else {
        return Err(XsdError::MissingElement {
            name: "restriction".to_string(),
            parent: node.name().to_string(),
            range: node.range(),
        });
    };

    children.prevent_unvisited_children()?;

    Ok(content)
}
