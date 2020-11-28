use crate::ast::{Attribute, LeafContent, LeafDefinition, Name, Namespace, Root};
use crate::xsd::context::{Context, NS_XSD};
use crate::xsd::node::Node;
use crate::xsd::XsdError;

pub fn parse<'a, 'input>(
    node: Node<'a, 'input>,
    parent: &Name,
    ctx: &Context<'a, 'input>,
) -> Result<Option<Attribute>, XsdError>
where
    'a: 'input,
{
    let name = Name::new(node.try_attribute("name")?.value(), Namespace::None);

    let mut children = node.children().namespace(NS_XSD).collect();
    let docs = children
        .remove("annotation", Some(NS_XSD))
        .map(super::annotation::parse)
        .transpose()?
        .flatten();

    let content = if let Some(child) = children.remove("simpleType", Some(NS_XSD)) {
        let root = super::simple_type::parse(child, ctx)?;
        if let Root::Leaf(LeafDefinition {
            content: LeafContent::Literal(content),
            ..
        }) = root
        {
            // NOTEN: flatten the type is only fine as long as we don't have sepcial handling
            // for restrictions
            LeafContent::Literal(content)
        } else {
            let virtual_name = super::derive_virtual_name(
                vec![parent, &name, &ctx.get_node_name("Data", false)],
                ctx,
                false,
            );
            ctx.add_root(virtual_name.clone(), root);

            LeafContent::Named(virtual_name)
        }
    } else {
        let type_attr = node.try_attribute("type")?;
        let content = ctx.get_type_name(&type_attr)?;
        if let LeafContent::Named(name) = &content {
            ctx.discover_type(&name);
        }
        content
    };

    children.prevent_unvisited_children()?;

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
        docs,
    }))
}
