use std::str::FromStr;

use rust_decimal::Decimal;

use crate::ast::{LeafContent, LeafDefinition, Name, Namespace, Restriction, Root};
use crate::xsd::context::{Context, NS_XSD};
use crate::xsd::error::XsdError;
use crate::xsd::node::Node;

pub fn parse<'a, 'input>(
    node: Node<'a, 'input>,
    ctx: &Context<'a, 'input>,
) -> Result<Root, XsdError>
where
    'a: 'input,
{
    node.prevent_unvisited_attributes()?;

    let mut children = node.children().namespace(NS_XSD).collect();
    let docs = super::parse_annotation(children.remove("annotation", Some(NS_XSD)))?;

    let restriction = children.try_remove("restriction", Some(NS_XSD))?;

    let attr = restriction.try_attribute("base")?;
    let type_name = ctx.get_type_name(&attr)?;
    let type_ = match type_name {
        LeafContent::Literal(type_) => type_,
        LeafContent::Named(_) | LeafContent::Fixed(_) => {
            return Err(XsdError::UnsupportedAttributeValue {
                name: "base".to_string(),
                value: attr.value().to_string(),
                element: "restriction".to_string(),
                range: attr.range(),
            });
        }
    };

    children.prevent_unvisited_children()?;

    let mut restrictions = Vec::new();
    let mut enumerations = Vec::new();

    for child in restriction.children().namespace(NS_XSD).iter() {
        match child.name() {
            "enumeration" => {
                enumerations.push(Name::new(
                    child.try_attribute("value")?.value().into_owned(),
                    Namespace::None,
                ));
            }
            "length" => {
                let attr = child.try_attribute("value")?;
                let value = usize::from_str(&attr.value()).map_err(|err| XsdError::ParseInt {
                    err,
                    range: attr.range(),
                })?;
                restrictions.push(Restriction::Length(value));
            }
            "minLength" => {
                let attr = child.try_attribute("value")?;
                let value = usize::from_str(&attr.value()).map_err(|err| XsdError::ParseInt {
                    err,
                    range: attr.range(),
                })?;
                restrictions.push(Restriction::MinLength(value));
            }
            "maxLength" => {
                let attr = child.try_attribute("value")?;
                let value = usize::from_str(&attr.value()).map_err(|err| XsdError::ParseInt {
                    err,
                    range: attr.range(),
                })?;
                restrictions.push(Restriction::MaxLength(value));
            }
            "pattern" => restrictions.push(Restriction::Pattern(
                child.try_attribute("value")?.value().into_owned(),
            )),
            "minInclusive" => {
                let attr = child.try_attribute("value")?;
                let value =
                    Decimal::from_str(&attr.value()).map_err(|err| XsdError::ParseDecimal {
                        err,
                        range: attr.range(),
                    })?;
                restrictions.push(Restriction::MinInclusive(value));
            }
            "maxInclusive" => {
                let attr = child.try_attribute("value")?;
                let value =
                    Decimal::from_str(&attr.value()).map_err(|err| XsdError::ParseDecimal {
                        err,
                        range: attr.range(),
                    })?;
                restrictions.push(Restriction::MaxInclusive(value));
            }
            "fractionDigits" => {
                let attr = child.try_attribute("value")?;
                let value = usize::from_str(&attr.value()).map_err(|err| XsdError::ParseInt {
                    err,
                    range: attr.range(),
                })?;
                restrictions.push(Restriction::FractionDigits(value));
            }
            "totalDigits" => {
                let attr = child.try_attribute("value")?;
                let value = usize::from_str(&attr.value()).map_err(|err| XsdError::ParseInt {
                    err,
                    range: attr.range(),
                })?;
                restrictions.push(Restriction::TotalDigits(value));
            }
            child_name => {
                return Err(XsdError::UnsupportedElement {
                    name: child_name.to_string(),
                    range: child.range(),
                })
            }
        }
    }

    Ok(if enumerations.is_empty() {
        Root::Leaf(LeafDefinition {
            content: LeafContent::Literal(type_),
            restrictions,
            docs,
        })
    } else {
        Root::Enum(enumerations)
    })
}
