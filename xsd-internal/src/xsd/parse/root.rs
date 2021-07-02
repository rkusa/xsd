use crate::ast::{Name, Root};
use crate::xsd::context::Context;
use crate::xsd::error::XsdError;
use crate::xsd::node::Node;

pub fn parse<'a, 'input>(
    node: Node<'a, 'input>,
    parent: &Name,
    ctx: &Context<'a, 'input>,
) -> Result<Root, XsdError>
where
    'a: 'input,
{
    match node.name() {
        "element" => super::element::parse_root(node, ctx),
        "complexType" => super::complex_type::parse(node, parent, ctx),
        "simpleType" => super::simple_type::parse(node, ctx),
        child_name => Err(XsdError::UnsupportedElement {
            name: child_name.to_string(),
            range: node.range(),
        }),
    }
}
