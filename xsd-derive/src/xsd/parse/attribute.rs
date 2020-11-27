use crate::ast::{Attribute, LeafContent, Name, Namespace};
use crate::xsd::context::{Context, NS_XSD};
use crate::xsd::node::Node;
use crate::xsd::XsdError;

pub fn parse<'a, 'input>(
    node: Node<'a, 'input>,
    ctx: &mut Context<'a, 'input>,
) -> Result<Option<Attribute>, XsdError>
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
        ctx.discover_type(&name);
    }

    let default = node.attribute("default").map(|a| a.value().to_string());
    let is_optional = match node.attribute("use").map(|attr| attr.value()).as_deref() {
        Some("required") => false,
        Some("optional") | None => true,
        Some("prohibited") => return Ok(None),
        Some(val) => {
            return Err(XsdError::UnsupportedAttributeValue {
                name: "use".to_string(),
                value: val.to_owned(),
                element: node.name().to_string(),
                range: node.range(),
            })
        }
    };
    if default.is_some() && !is_optional {
        return Err(XsdError::UnexpectedAttribute {
            name: "default".to_string(),
            element: node.name().to_string(),
            range: node.range(),
        });
    }

    node.prevent_unvisited_attributes()?;

    Ok(Some(Attribute {
        name,
        content,
        default,
        is_optional,
    }))
}
