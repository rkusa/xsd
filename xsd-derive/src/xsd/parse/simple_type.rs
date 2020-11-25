use crate::ast::{LeafContent, LeafDefinition, Restriction};
use crate::xsd::context::{Context, NS_XSD};
use crate::xsd::node::Node;
use crate::xsd::XsdError;

pub fn parse<'a, 'input>(
    node: Node<'a, 'input>,
    ctx: &mut Context<'a, 'input>,
) -> Result<LeafDefinition, XsdError>
where
    'a: 'input,
{
    let mut children = node.children().namespace(NS_XSD).collect();
    let restriction = children.try_remove("restriction", Some(NS_XSD))?;

    let attr = restriction.try_attribute("base")?;
    let type_name = ctx.get_type_name(&attr)?;
    let type_ = match type_name {
        LeafContent::Literal(type_) => type_,
        LeafContent::Named(_) => {
            return Err(XsdError::UnsupportedAttributeValue {
                name: "base".to_string(),
                value: attr.value().to_string(),
                element: "restriction".to_string(),
                range: attr.range(),
            });
        }
    };

    children.prevent_unvisited_children()?;

    let mut restrictions = Vec::new();
    let mut enumerations = Vec::new();

    for child in restriction.children().namespace(NS_XSD).iter() {
        match child.name() {
            "enumeration" => {
                enumerations.push(child.try_attribute("value")?.value().into_owned());
            }
            child_name => {
                return Err(XsdError::UnsupportedElement {
                    name: child_name.to_string(),
                    parent: node.name().to_string(),
                    range: child.range(),
                })
            }
        }
    }

    if !enumerations.is_empty() {
        restrictions.push(Restriction::Enum(enumerations));
    }

    Ok(LeafDefinition {
        content: LeafContent::Literal(type_),
        restrictions,
    })
}
