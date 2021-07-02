use std::str::FromStr;

use crate::ast::{
    Leaf, LeafContent, LeafDefinition, LiteralType, MaxOccurs, MinOccurs, Name, Root,
};
use crate::xsd::context::{Context, NS_XSD};
use crate::xsd::error::XsdError;
use crate::xsd::node::{Attribute, Node};

pub fn parse_root<'a, 'input>(
    node: Node<'a, 'input>,
    ctx: &Context<'a, 'input>,
) -> Result<Root, XsdError>
where
    'a: 'input,
{
    let mut children = node.children().namespace(NS_XSD).collect();
    let docs = super::parse_annotation(children.remove("annotation", Some(NS_XSD)))?;

    // <element type="xs:string" /> | <element type="MyCustomType" />
    if let Some(attr) = node.attribute("type") {
        let mut content = ctx.get_type_name(attr)?;
        match &mut content {
            LeafContent::Named(name) => ctx.discover_type(name, None),
            content @ LeafContent::Literal(_) => {
                if let Some(attr) = node.attribute("fixed") {
                    *content = LeafContent::Fixed(attr.value().to_string());
                }
            }
            _ => {}
        }

        node.prevent_unvisited_attributes()?;
        children.prevent_unvisited_children()?;

        Ok(Root::Leaf(LeafDefinition {
            content,
            restrictions: Vec::new(),
            docs,
        }))
    } else {
        node.prevent_unvisited_attributes()?;

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

        Ok(result.with_docs(docs))
    }
}

pub fn parse_child<'a, 'input>(
    node: Node<'a, 'input>,
    parent: &Name,
    ctx: &Context<'a, 'input>,
) -> Result<Leaf, XsdError>
where
    'a: 'input,
{
    let name = ctx.get_node_name(&node.try_attribute("name")?.value(), false);

    let min_occurs = parse_min_occurs(node.attribute("minOccurs"))?;
    let max_occurs = parse_max_occurs(node.attribute("maxOccurs"))?;

    // mark `default` attribute as visited
    node.attribute("default");

    // TODO: implement attribute?
    node.attribute("form");

    let docs = super::parse_annotation(node.child("annotation", Some(NS_XSD)))?;

    // <element type="xs:string" /> | <element type="MyCustomType" />
    if let Some(attr) = node.attribute("type") {
        let mut content = ctx.get_type_name(attr)?;
        match &mut content {
            LeafContent::Named(name) => ctx.discover_type(name, Some(parent)),
            content @ LeafContent::Literal(_) => {
                if let Some(attr) = node.attribute("fixed") {
                    *content = LeafContent::Fixed(attr.value().to_string());
                }
            }
            _ => {}
        }

        node.prevent_unvisited_attributes()?;

        Ok(Leaf {
            name,
            definition: LeafDefinition {
                content,
                restrictions: Vec::new(),
                docs,
            },
            is_virtual: false,
            min_occurs,
            max_occurs,
        })
    } else if node.child("simpleType", Some(NS_XSD)).is_none()
        && node.child("complexType", Some(NS_XSD)).is_none()
    {
        node.prevent_unvisited_attributes()?;

        // TODO: add test for that case
        Ok(Leaf {
            name,
            definition: LeafDefinition {
                content: LeafContent::Literal(LiteralType::Any),
                restrictions: Vec::new(),
                docs,
            },
            is_virtual: false,
            min_occurs,
            max_occurs,
        })
    } else {
        node.prevent_unvisited_attributes()?;

        // create a new virtual type
        let virtual_name = super::derive_virtual_name(
            vec![parent, &name, &ctx.get_node_name("Data", false)],
            ctx,
            false,
        );
        ctx.add_root(
            virtual_name.clone(),
            super::root::parse(node, &virtual_name, ctx)?,
        );
        Ok(Leaf {
            name,
            definition: LeafDefinition {
                content: LeafContent::Named(virtual_name),
                restrictions: Vec::new(),
                docs,
            },
            is_virtual: false,
            min_occurs,
            max_occurs,
        })
    }
}

pub fn parse_min_occurs(occurs: Option<&Attribute<'_, '_>>) -> Result<MinOccurs, XsdError> {
    match occurs {
        Some(attr) => Ok(MinOccurs(u32::from_str(&attr.value()).map_err(|err| {
            XsdError::ParseInt {
                err,
                range: attr.range(),
            }
        })?)),
        None => Ok(MinOccurs::default()),
    }
}

pub fn parse_max_occurs(occurs: Option<&Attribute<'_, '_>>) -> Result<MaxOccurs, XsdError> {
    match occurs {
        Some(attr) => {
            let val = attr.value();
            if &val == "unbounded" {
                Ok(MaxOccurs::Unbounded)
            } else {
                Ok(MaxOccurs::Number(u32::from_str(&val).map_err(|err| {
                    XsdError::ParseInt {
                        err,
                        range: attr.range(),
                    }
                })?))
            }
        }
        None => Ok(MaxOccurs::default()),
    }
}
