use crate::ast::{Attribute, LeafContent, Name, Namespace};
use crate::xsd::context::{Context, NS_XSD};
use crate::xsd::node::Node;
use crate::xsd::XsdError;

pub fn parse<'a, 'input>(
    node: Node<'a, 'input>,
    ctx: &mut Context<'a, 'input>,
) -> Result<Attribute, XsdError>
where
    'a: 'input,
{
    let name = Name::new(node.try_attribute("name")?.value(), Namespace::None);

    let mut children = node.children().namespace(NS_XSD).collect();
    // TODO: actually use docs
    let _docs = children
        .remove("annotation", Some(NS_XSD))
        .map(super::annotation::parse)
        .transpose()?;

    children.prevent_unvisited_children()?;

    let type_attr = node.try_attribute("type")?;
    let content = match ctx.get_type_name(&type_attr)? {
        LeafContent::Literal(literal) => literal,
        LeafContent::Named(_) => {
            return Err(XsdError::UnsupportedAttributeValue {
                name: "type".to_string(),
                value: type_attr.value().to_string(),
                element: node.name().to_string(),
                range: type_attr.range(),
            })
        }
    };

    Ok(Attribute { name, content })
}
