use crate::types::{ElementContent, ElementDefinition, LeafContent};
use crate::xsd::context::{Context, NS_XSD};
use crate::xsd::node::Node;
use crate::xsd::XsdError;

pub fn parse(node: &Node<'_, '_>, ctx: &Context<'_, '_>) -> Result<ElementDefinition, XsdError> {
    if let Some(attr) = node.attribute("type") {
        let type_name = ctx.get_type_name(attr)?;

        match type_name {
            LeafContent::Literal(literal) => Ok(ElementDefinition {
                content: ElementContent::Literal(literal),
            }),
            LeafContent::Named(_name) => {
                unimplemented!("<element type='non xs' />")
            }
        }
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
                        Ok((Some(super::complex_type::parse(&child, ctx)?), docs))
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

        Ok(ElementDefinition { content })
    }
}
