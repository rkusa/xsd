use crate::types::{ElementContent, Name};
use crate::xsd::context::{Context, NS_XSD};
use crate::xsd::node::Node;
use crate::xsd::XsdError;

pub fn parse<'a, 'input>(
    node: Node<'a, 'input>,
    parent: &Name,
    ctx: &mut Context<'a, 'input>,
) -> Result<ElementContent, XsdError>
where
    'a: 'input,
{
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
                    Ok(Some(super::sequence::parse(child, parent, ctx)?))
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
