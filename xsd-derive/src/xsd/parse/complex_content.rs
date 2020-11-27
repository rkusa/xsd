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
    node.prevent_unvisited_attributes()?;

    let mut children = node.children().namespace(NS_XSD).collect();
    let extension = children.try_remove("extension", Some(NS_XSD))?;
    children.prevent_unvisited_children()?;

    let attr = extension.try_attribute("base")?;
    let content = ctx.get_type_name(&attr)?;
    match &content {
        LeafContent::Literal(_) => {
            return Err(XsdError::UnsupportedAttributeValue {
                name: "base".to_string(),
                value: attr.value().to_string(),
                element: "extension".to_string(),
                range: attr.range(),
            })
        }
        LeafContent::Named(name) => ctx.discover_type(name),
    }

    let mut children = extension.children().namespace(NS_XSD).collect();

    // read all attributes
    let mut attributes = Vec::new();
    while let Some(child) = children.remove("attribute", Some(NS_XSD)) {
        if let Some(attr) = super::attribute::parse(child, ctx)? {
            attributes.push(attr);
        }
    }

    children.prevent_unvisited_children()?;

    // TODO: merge with extension instead of having it as `value` property?
    Ok(ElementDefinition {
        attributes,
        content: Some(ElementContent::Leaf(LeafDefinition {
            content,
            restrictions: Vec::new(),
        })),
        is_virtual: false,
    })
}
