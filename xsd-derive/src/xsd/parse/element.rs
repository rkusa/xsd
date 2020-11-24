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
                content: ElementContent::Literal(literal),
            }),
            LeafContent::Named(_name) => {
                unimplemented!("<element type='non xs' />")
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
            content: ElementContent::Reference(name),
        })
    } else {
        let (content, _docs) = node.children().namespace(NS_XSD).iter().try_fold(
            (None, None),
            |(content, docs), child| match child.name() {
                "annotation" => {
                    if docs.is_some() {
                        Err(XsdError::MultipleTypes {
                            name: child.name().to_string(),
                            range: child.range(),
                        })
                    } else {
                        Ok((content, super::annotation::parse(&child)?))
                    }
                }
                "complexType" => {
                    if content.is_some() {
                        Err(XsdError::MultipleTypes {
                            name: child.name().to_string(),
                            range: child.range(),
                        })
                    } else {
                        let name = node.try_attribute("name")?.value();
                        let name = ctx.get_node_name(&name, false);
                        Ok((Some(super::complex_type::parse(child, &name, ctx)?), docs))
                    }
                }
                "simpleType" => {
                    if content.is_some() {
                        Err(XsdError::MultipleTypes {
                            name: child.name().to_string(),
                            range: child.range(),
                        })
                    } else {
                        Ok((Some(super::simple_type::parse(child, ctx)?), docs))
                    }
                }
                child_name => Err(XsdError::UnsupportedElement {
                    name: child_name.to_string(),
                    parent: node.name().to_string(),
                    range: child.range(),
                }),
            },
        )?;

        let content = content.ok_or_else(|| XsdError::MissingElement {
            name: "simpleType|complexType".to_string(),
            parent: "element".to_string(),
            range: node.range(),
        })?;

        Ok(ElementDefinition { kind, content })
    }
}
