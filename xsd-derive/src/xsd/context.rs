use std::collections::HashMap;

use super::lazy::Lazy;
use super::node::{Attribute, Node};
use super::XsdError;
use crate::ast::{Kind, LeafContent, LiteralType, Name, Namespace};

pub struct Context<'a, 'input> {
    simple_types: HashMap<Name, Lazy<'a, 'input>>,
    complex_types: HashMap<Name, Lazy<'a, 'input>>,
    elements: HashMap<Name, Lazy<'a, 'input>>,
    default_namespace: Option<&'input str>,
    target_namespace: Option<&'a str>,
    namespaces: HashMap<&'input str, &'input str>,
    is_qualified: bool,
}

pub const NS_XSD: &str = "http://www.w3.org/2001/XMLSchema";

impl<'a, 'input> Context<'a, 'input>
where
    'a: 'input,
{
    pub fn new(schema: &roxmltree::Node<'a, 'input>, target_namespace: Option<&'a str>) -> Self {
        let mut namespaces = HashMap::new();
        for ns in schema.namespaces() {
            if let Some(prefix) = ns.name() {
                namespaces.insert(prefix, ns.uri());
            }
        }
        Context {
            simple_types: HashMap::new(),
            complex_types: HashMap::new(),
            elements: HashMap::new(),
            default_namespace: schema.default_namespace(),
            target_namespace,
            namespaces,
            is_qualified: schema.attribute("elementFormDefault") == Some("qualified"),
        }
    }

    pub fn add_simple_type(&mut self, name: Name, node: Node<'a, 'input>, kind: Kind) {
        self.simple_types.insert(name, Lazy::new(node, kind));
    }

    pub fn add_complex_type(&mut self, name: Name, node: Node<'a, 'input>, kind: Kind) {
        self.complex_types.insert(name, Lazy::new(node, kind));
    }

    pub fn add_element(&mut self, name: Name, node: Node<'a, 'input>, kind: Kind) {
        self.elements.insert(name, Lazy::new(node, kind));
    }

    pub fn remove_elements(&mut self) -> impl Iterator<Item = (Name, Lazy<'a, 'input>)> {
        let elements = std::mem::take(&mut self.elements);
        elements.into_iter()
    }

    pub fn get_node_name(&self, name: &str, is_top_level: bool) -> Name {
        Name::new(
            name,
            if self.is_qualified || is_top_level {
                Namespace::Target
            } else {
                Namespace::None
            },
        )
    }

    pub fn get_type_name<'b, 'c>(&self, attr: &Attribute<'b, 'c>) -> Result<LeafContent, XsdError> {
        let type_name = attr.value();
        let mut parts = type_name.splitn(2, ':');

        let name = match (parts.next(), parts.next()) {
            (Some(prefix), Some(name)) => {
                let ns = self
                    .namespaces
                    .get(prefix)
                    .ok_or_else(|| XsdError::MissingNamespace {
                        prefix: prefix.to_string(),
                        range: attr.range(),
                    })?;
                return if *ns == NS_XSD {
                    Ok(LeafContent::Literal(literal_from_str(name).ok_or_else(
                        || XsdError::UnsupportedType {
                            name: name.to_string(),
                            range: attr.range(),
                        },
                    )?))
                } else if Some(*ns) == self.target_namespace {
                    Ok(LeafContent::Named(Name::new(name, Namespace::Target)))
                } else {
                    Ok(LeafContent::Named(Name::new(
                        name,
                        Namespace::Other(ns.to_string()),
                    )))
                };
            }
            (Some(name), None) => name,
            _ => &type_name,
        };
        if self.default_namespace == Some(NS_XSD) {
            Ok(LeafContent::Literal(literal_from_str(name).ok_or_else(
                || XsdError::UnsupportedType {
                    name: name.to_string(),
                    range: attr.range(),
                },
            )?))
        } else {
            Ok(LeafContent::Named(Name::new(name, Namespace::None)))
        }
    }
}

fn literal_from_str(literal: &str) -> Option<LiteralType> {
    Some(match literal {
        "string" => LiteralType::String,
        "boolean" => LiteralType::Boolean,
        "integer" | "long" => LiteralType::Int64,
        "nonNegativeInteger" => LiteralType::Uint64,
        // TODO: add validation for non zero?
        "positiveInteger" => LiteralType::Uint64,
        "int" => LiteralType::Int32,
        "decimal" => LiteralType::Decimal,
        "float" => LiteralType::Float32,
        "time" => LiteralType::Time,
        "date" => LiteralType::Date,
        "dateTime" => LiteralType::DateTime,
        "duration" => LiteralType::Duration,
        "base64Binary" => LiteralType::Base64Binary,
        "any" => LiteralType::Any,
        "anyType" => LiteralType::Any,
        _ => return None,
    })
}
