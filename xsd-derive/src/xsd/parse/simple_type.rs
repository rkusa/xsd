use crate::ast::Namespace::None;
use crate::ast::{LeafContent, LeafDefinition, MaxOccurs, MinOccurs, Name, Root};
use crate::xsd::context::{Context, NS_XSD};
use crate::xsd::node::Node;
use crate::xsd::XsdError;

pub fn parse<'a, 'input>(
    node: Node<'a, 'input>,
    ctx: &mut Context<'a, 'input>,
) -> Result<Root, XsdError>
where
    'a: 'input,
{
    node.prevent_unvisited_attributes()?;

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

    let restrictions = Vec::new();
    let mut enumerations = Vec::new();

    for child in restriction.children().namespace(NS_XSD).iter() {
        match child.name() {
            "enumeration" => {
                enumerations.push(Name::new(
                    child.try_attribute("value")?.value().into_owned(),
                    None,
                ));
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

    Ok(if enumerations.is_empty() {
        Root::Leaf(LeafDefinition {
            content: LeafContent::Literal(type_),
            restrictions,
            min_occurs: MinOccurs::default(),
            max_occurs: MaxOccurs::default(),
        })
    } else {
        Root::Enum(enumerations)
    })
}
