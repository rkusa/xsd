use crate::types::{ElementContent, ElementDefinition, Literal};
use crate::xsd::context::{Context, DataType};
use crate::xsd::node::Node;
use crate::xsd::XsdError;

pub fn parse(node: &Node<'_, '_>, ctx: &Context<'_, '_>) -> Result<ElementDefinition, XsdError> {
    if let Some(attr) = node.attribute("type") {
        let type_name = ctx.get_type_name(attr)?;

        match type_name {
            DataType::Literal(literal) => {
                return Ok(ElementDefinition {
                    content: ElementContent::Literal(Literal { type_: literal }),
                });
            }
            DataType::Named(name) => {
                unimplemented!("<element type='non xs' />")
            }
        }
    }

    unimplemented!("other elements")
}
