use crate::ast::{ElementContent, ElementDefinition, LeafContent, LeafDefinition};
use crate::xsd::context::{Context, NS_XSD};
use crate::xsd::node::Node;
use crate::xsd::XsdError;

pub fn parse<'a, 'input>(
    node: Node<'a, 'input>,
    ctx: &mut Context<'a, 'input>,
) -> Result<ElementDefinition, XsdError>
where
    'a: 'input,
{
    let mut children = node.children().namespace(NS_XSD).collect();
    let extension = children.try_remove("extension", Some(NS_XSD))?;
    children.prevent_unvisited_children()?;

    let attr = extension.try_attribute("base")?;
    let type_name = ctx.get_type_name(&attr)?;
    let type_ = match type_name {
        LeafContent::Literal(type_) => type_,
        LeafContent::Named(_) => {
            return Err(XsdError::UnsupportedAttributeValue {
                name: "base".to_string(),
                value: attr.value().to_string(),
                element: "extension".to_string(),
                range: attr.range(),
            });
        }
    };

    let mut children = extension.children().namespace(NS_XSD).collect();

    // read all attributes
    let mut attributes = Vec::new();
    while let Some(child) = children.remove("attribute", Some(NS_XSD)) {
        attributes.push(super::attribute::parse(child, ctx)?);
    }

    children.prevent_unvisited_children()?;

    Ok(ElementDefinition {
        attributes,
        content: Some(ElementContent::Leaf(LeafDefinition {
            content: LeafContent::Literal(type_),
            restrictions: Vec::new(),
        })),
    })
}
