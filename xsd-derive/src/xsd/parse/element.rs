use crate::ast::{LeafContent, LeafDefinition, Name, Root};
use crate::xsd::context::{Context, NS_XSD};
use crate::xsd::node::Node;
use crate::xsd::XsdError;

pub fn parse_root<'a, 'input>(
    node: Node<'a, 'input>,
    ctx: &mut Context<'a, 'input>,
) -> Result<Root, XsdError>
where
    'a: 'input,
{
    // <element type="xs:string" /> | <element type="MyCustomType" />
    if let Some(attr) = node.attribute("type") {
        node.prevent_unvisited_attributes()?;
        let content = ctx.get_type_name(attr)?;
        if let LeafContent::Named(name) = &content {
            ctx.discover_type(name);
        }
        Ok(Root::Leaf(LeafDefinition {
            content,
            restrictions: Vec::new(),
        }))
    } else {
        node.prevent_unvisited_attributes()?;
        let mut children = node.children().namespace(NS_XSD).collect();
        // TODO: actually use docs
        let _docs = children
            .remove("annotation", Some(NS_XSD))
            .map(super::annotation::parse)
            .transpose()?;

        let result = if let Some(child) = children.remove("complexType", Some(NS_XSD)) {
            let name = node.try_attribute("name")?.value();
            let name = ctx.get_node_name(&name, false);
            super::complex_type::parse(child, &name, ctx)?
        } else if let Some(child) = children.remove("simpleType", Some(NS_XSD)) {
            super::simple_type::parse(child, ctx)?
        } else {
            return Err(XsdError::MissingElement {
                name: "simpleType|complexType".to_string(),
                parent: node.name().to_string(),
                range: node.range(),
            });
        };

        children.prevent_unvisited_children()?;

        Ok(result)
    }
}

pub fn parse_child<'a, 'input>(
    node: Node<'a, 'input>,
    parent: &Name,
    ctx: &mut Context<'a, 'input>,
) -> Result<LeafDefinition, XsdError>
where
    'a: 'input,
{
    // <element type="xs:string" /> | <element type="MyCustomType" />
    if let Some(attr) = node.attribute("type") {
        let content = ctx.get_type_name(attr)?;
        if let LeafContent::Named(name) = &content {
            ctx.discover_type(name);
        }
        Ok(LeafDefinition {
            content,
            restrictions: Vec::new(),
        })
    } else {
        // create a new virtual type
        let name = node.try_attribute("name")?.value();
        let mut virtual_name = parent.name.to_string();
        if !name.is_empty() {
            virtual_name += (&name[0..1]).to_ascii_uppercase().as_str();
            virtual_name += &name[1..];
        }
        let name = ctx.get_node_name(&virtual_name, false);
        ctx.add_element(name.clone(), node);
        Ok(LeafDefinition {
            content: LeafContent::Named(name),
            restrictions: Vec::new(),
        })
    }
}
