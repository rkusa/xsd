use crate::ast::{ElementContent, LeafContent};
use crate::xsd::context::{Context, NS_XSD};
use crate::xsd::node::Node;
use crate::xsd::XsdError;

pub fn parse<'a, 'input>(
    node: Node<'a, 'input>,
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
            "restriction" => {
                if content.is_some() {
                    Err(XsdError::MultipleTypes {
                        name: child.name().to_string(),
                        range: child.range(),
                    })
                } else {
                    let attr = child.try_attribute("base")?;
                    let type_name = ctx.get_type_name(&attr)?;
                    Ok(Some(match type_name {
                        LeafContent::Literal(literal) => ElementContent::Literal(literal),
                        LeafContent::Named(_) => {
                            return Err(XsdError::UnsupportedAttributeValue {
                                name: "base".to_string(),
                                value: attr.value().to_string(),
                                element: "restriction".to_string(),
                                range: attr.range(),
                            });
                        }
                    }))
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
