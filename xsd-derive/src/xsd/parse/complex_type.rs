use crate::types::ElementContent;
use crate::xsd::context::{Context, NS_XSD};
use crate::xsd::node::Node;
use crate::xsd::XsdError;

pub fn parse(node: &Node<'_, '_>, ctx: &Context<'_, '_>) -> Result<ElementContent, XsdError> {
    let content = node
        .children()
        .namespace(NS_XSD)
        .iter()
        .try_fold(None, |content, child| match child.name() {
            "sequence" => {
                if content.is_some() {
                    Err(XsdError::MultipleTypes {
                        name: child.name().to_string(),
                        range: child.range(),
                    })
                } else {
                    Ok(Some(super::sequence::parse(&child, ctx)?))
                }
            }
            child_name => Err(XsdError::UnsupportedElement {
                name: child_name.to_string(),
                parent: node.name().to_string(),
                range: child.range(),
            }),
        })?;

    let content = content.ok_or_else(|| XsdError::MissingElement {
        name: "sequence".to_string(),
        parent: "element".to_string(),
        range: node.range(),
    })?;

    Ok(content)
}
