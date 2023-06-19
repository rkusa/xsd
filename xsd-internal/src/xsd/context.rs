use std::collections::{HashMap, HashSet};

use super::error::XsdError;
use super::node::Attribute;
use super::schema::Schema;
use crate::ast::{LeafContent, LeafDefinition, LiteralType, Name, Namespace, Namespaces, Root};
use proc_macro2::TokenStream;
use quote::quote;

/// The context that holds all data that is either necessary during parsing a schema, or is
/// pupulated during parsing a scheme with data necessary for the subsequent code-generation.
pub struct Context<'input> {
    roots: HashMap<Name, Root>,
    default_namespace: Option<&'input str>,
    target_namespace: Namespace,
    document_namespaces: HashMap<&'input str, &'input str>,
    is_qualified: bool,
    shared: SharedContext,
}

/// The parts of the [Context] that are shared between schema files (during includes/imports).
#[derive(Debug, Default)]
pub struct SharedContext {
    pub namespaces: Namespaces,
    /// Dependencies between structs. Key = parent, Value = child
    pub dependencies: HashMap<Name, HashSet<Name>>,
}

/// The context reduced to the data necessary for the code-generation.
#[derive(Debug)]
pub struct SchemaContext {
    pub elements: HashMap<Name, Root>,
    pub target_namespace: Namespace,
    pub is_qualified: bool,
    pub namespaces: Namespaces,
}

pub const NS_XSD: &str = "http://www.w3.org/2001/XMLSchema";

impl<'input> Context<'input> {
    pub fn new<'a: 'input>(
        schema: &'a roxmltree::Node<'a, 'input>,
        target_namespace: Option<&'a str>,
        mut shared: SharedContext,
    ) -> Self {
        let mut document_namespaces = HashMap::new();
        for ns in schema.namespaces() {
            if let Some(prefix) = ns.name() {
                document_namespaces.insert(prefix, ns.uri());
            }
        }
        Context {
            roots: Default::default(),
            default_namespace: schema.default_namespace(),
            target_namespace: target_namespace
                .map(|tn| shared.namespaces.get_or_insert(tn))
                .unwrap_or_default(),
            document_namespaces,
            is_qualified: schema.attribute("elementFormDefault") == Some("qualified"),
            shared,
        }
    }

    pub fn take_shared(&mut self) -> SharedContext {
        std::mem::take(&mut self.shared)
    }
    pub fn set_shared(&mut self, shared: SharedContext) {
        self.shared = shared
    }

    pub fn add_root(&mut self, name: Name, root: Root) {
        self.roots.insert(name, root);
    }

    pub fn into_schema(self) -> Schema {
        Schema {
            context: SchemaContext {
                target_namespace: self.target_namespace(),
                elements: self.roots,
                is_qualified: self.is_qualified,
                namespaces: self.shared.namespaces,
            },
            dependencies: self.shared.dependencies,
        }
    }

    pub fn discover_type(&mut self, name: &Name, parent: Option<&Name>) {
        if let Some(parent) = parent {
            let dependends = self.shared.dependencies.entry(parent.clone()).or_default();
            dependends.insert(name.clone());
        }
    }

    pub fn get_node_name(&self, name: &str, is_top_level: bool) -> Name {
        Name::new(
            name,
            if self.is_qualified || is_top_level {
                self.target_namespace()
            } else {
                Namespace::None
            },
        )
    }

    pub fn get_type_name(&mut self, attr: &Attribute<'_, '_>) -> Result<LeafContent, XsdError> {
        let type_name = attr.value();
        let mut parts = type_name.splitn(2, ':');

        let name = match (parts.next(), parts.next()) {
            (Some(prefix), Some(name)) => {
                let ns = self.document_namespaces.get(prefix).ok_or_else(|| {
                    XsdError::MissingNamespace {
                        prefix: prefix.to_string(),
                        range: attr.range(),
                    }
                })?;
                return if *ns == NS_XSD {
                    Ok(LeafContent::Literal(literal_from_str(name).ok_or_else(
                        || XsdError::UnsupportedType {
                            name: name.to_string(),
                            range: attr.range(),
                        },
                    )?))
                } else {
                    Ok(LeafContent::Named(Name::new(
                        name,
                        self.shared.namespaces.get_or_insert(ns),
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

    pub fn target_namespace(&self) -> Namespace {
        self.target_namespace
    }
}

impl SchemaContext {
    pub fn get_xml_element_name(&self, name: &Name) -> String {
        match &name.namespace {
            Namespace::None => name.name.clone(),
            Namespace::Id(id) => {
                if self.is_qualified && name.namespace == self.target_namespace {
                    name.name.clone()
                } else {
                    let ns = self.namespaces.get_by_id(*id);
                    format!("{}:{}", ns.prefix, name.name)
                }
            }
        }
    }

    pub fn quote_xml_namespace(&self, name: &Name) -> TokenStream {
        match &name.namespace {
            Namespace::None => quote!(None),
            Namespace::Id(id) => {
                let ns = self.namespaces.get_by_id(*id);
                let ns = &ns.namespace;
                quote!(Some(#ns))
            }
        }
    }

    pub fn resolve(&self, name: &Name) -> Option<&Root> {
        let mut next = self.elements.get(name);
        while let Some(Root::Leaf(LeafDefinition {
            content: LeafContent::Named(name),
            ..
        })) = next
        {
            next = self.elements.get(name);
        }
        next
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
        "hexBinary" => LiteralType::HexBinary,
        "any" => LiteralType::Any,
        "anyType" => LiteralType::Any,
        _ => return None,
    })
}
