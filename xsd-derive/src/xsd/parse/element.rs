use crate::ast::{ElementContent, ElementDefinition, Kind, LeafContent, Name};
use crate::xsd::context::{Context, NS_XSD};
use crate::xsd::node::Node;
use crate::xsd::XsdError;

pub fn parse<'a, 'input>(
    node: Node<'a, 'input>,
    parent: &Name,
    ctx: &mut Context<'a, 'input>,
    kind: Kind,
) -> Result<ElementDefinition, XsdError>
where
    'a: 'input,
{
    if let Some(attr) = node.attribute("type") {
        let type_name = ctx.get_type_name(attr)?;

        match type_name {
            LeafContent::Literal(literal) => Ok(ElementDefinition {
                kind,
                attributes: Vec::new(),
                content: ElementContent::Literal(literal),
            }),
            LeafContent::Named(name) => {
                ctx.discover_type(&name);
                Ok(ElementDefinition {
                    kind: Kind::Root,
                    attributes: Vec::new(),
                    content: ElementContent::Reference(name),
                })
            }
        }
    } else if kind == Kind::Child {
        // create a new virtual type
        let name = node.try_attribute("name")?.value();
        let mut virtual_name = parent.name.to_string();
        if !name.is_empty() {
            virtual_name += (&name[0..1]).to_ascii_uppercase().as_str();
            virtual_name += &name[1..];
        }
        let name = ctx.get_node_name(&virtual_name, false);
        ctx.add_element(name.clone(), node, Kind::Virtual);
        Ok(ElementDefinition {
            kind: Kind::Virtual,
            attributes: Vec::new(),
            content: ElementContent::Reference(name),
        })
    } else {
        let mut children = node.children().namespace(NS_XSD).collect();
        // TODO: actually use docs
        let _docs = children
            .remove("annotation", Some(NS_XSD))
            .map(super::annotation::parse)
            .transpose()?;

        let definition = if let Some(child) = children.remove("complexType", Some(NS_XSD)) {
            let name = node.try_attribute("name")?.value();
            let name = ctx.get_node_name(&name, false);
            super::complex_type::parse(child, &name, ctx, kind)?
        } else if let Some(child) = children.remove("simpleType", Some(NS_XSD)) {
            ElementDefinition {
                kind,
                attributes: Vec::new(),
                content: super::simple_type::parse(child, ctx)?,
            }
        } else {
            return Err(XsdError::MissingElement {
                name: "simpleType|complexType".to_string(),
                parent: node.name().to_string(),
                range: node.range(),
            });
        };

        children.prevent_unvisited_children()?;

        Ok(definition)
    }
}
