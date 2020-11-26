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
    let content = ctx.get_type_name(&type_attr)?;
    if let LeafContent::Named(name) = &content {
        if !ctx.discover_simple_type(&name) {
            return Err(XsdError::MissingSimpleType {
                name: name.name.to_string(),
                range: type_attr.range(),
            });
        }
    }

    Ok(Attribute { name, content })
}
