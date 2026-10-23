use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use super::error::XsdError;
use super::node::Attribute;
use super::schema::Schema;
use crate::ast::{LeafContent, LeafDefinition, LiteralType, Name, Namespace, Namespaces, Root};
use crate::utils::escape_ident;
use inflector::Inflector;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

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
    /// Anonymous types (in the order they were discovered). Their final names are only
    /// determined once the whole schema is known (see [resolve_type_names]).
    pub anonymous_types: Vec<(Name, AnonymousType)>,
    /// The schema files that have already been parsed (see [LoadedSchema]).
    pub loaded_schemas: HashSet<LoadedSchema>,
}

/// A schema file that has been included or imported. Only parsing it once prevents duplicates of
/// its anonymous types (which would then need to be numbered) and endless circular imports.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LoadedSchema {
    pub path: PathBuf,
    /// The namespace of the including schema for `<xs:include>`, as an included schema without a
    /// target namespace takes on the one of the schema including it ("chameleon include").
    pub include_namespace: Option<Option<String>>,
}

/// A type without a name in the schema (e.g. an inline `<xs:complexType>` of an element, or a
/// `<xs:choice>` inside of a `<xs:sequence>`).
#[derive(Debug, Clone)]
pub struct AnonymousType {
    /// The type the anonymous type is defined in.
    pub parent: Name,
    /// The name of the anonymous type relative to its parent (e.g. the element name).
    pub segment: String,
    /// Whether the final name must at least contain the parent's name (e.g. for groups like
    /// `<xs:choice>`, whose segment alone isn't descriptive enough).
    pub include_parent: bool,
    /// Appended to the name, if it still collides with another type after the type's whole path
    /// has been used.
    pub fallback_suffix: Option<&'static str>,
    /// Decides which of the types with the same path keeps the name, while the others get their
    /// fallback suffix (lowest wins): types (0), then groups (1), then sequences inside of
    /// choices (2).
    pub priority: u8,
}

/// The context reduced to the data necessary for the code-generation.
#[derive(Debug)]
pub struct SchemaContext {
    pub elements: HashMap<Name, Root>,
    pub target_namespace: Namespace,
    pub is_qualified: bool,
    pub namespaces: Namespaces,
    /// The final (Rust) names of all anonymous types.
    pub type_names: HashMap<Name, String>,
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

    pub fn shared_mut(&mut self) -> &mut SharedContext {
        &mut self.shared
    }

    pub fn add_root(&mut self, name: Name, root: Root) {
        self.roots.insert(name, root);
    }

    pub fn into_schema(self) -> Schema {
        let type_names = resolve_type_names(&self.roots, &self.shared.anonymous_types);
        Schema {
            context: SchemaContext {
                target_namespace: self.target_namespace(),
                elements: self.roots,
                is_qualified: self.is_qualified,
                namespaces: self.shared.namespaces,
                type_names,
            },
            dependencies: self.shared.dependencies,
            anonymous_types: self.shared.anonymous_types,
            loaded_schemas: self.shared.loaded_schemas,
        }
    }

    /// Registers a new anonymous type (e.g. the inline type of an element) defined in `parent`
    /// and returns its (internal) name. The final name is resolved once the whole schema is
    /// known.
    pub fn anonymous_type(&mut self, parent: &Name, segment: &str) -> Name {
        self.register_anonymous_type(parent, segment, false, None, 0)
    }

    /// Registers a new anonymous group (e.g. a `<xs:choice>` inside of a `<xs:sequence>`), whose
    /// name always contains its parent's name (e.g. `UserChoice`). The `fallback_suffix` (e.g.
    /// `Choice`) tells it apart from a type with the same path, like the type of the group's only
    /// element (whose name the group is derived from).
    pub fn anonymous_group(
        &mut self,
        parent: &Name,
        segment: &str,
        fallback_suffix: &'static str,
    ) -> Name {
        self.register_anonymous_type(parent, segment, true, Some(fallback_suffix), 1)
    }

    /// Registers a new anonymous `<xs:sequence>` inside of a `<xs:choice>`.
    pub fn anonymous_variant(&mut self, parent: &Name, segment: &str) -> Name {
        self.register_anonymous_type(parent, segment, true, Some("Variant"), 2)
    }

    fn register_anonymous_type(
        &mut self,
        parent: &Name,
        segment: &str,
        include_parent: bool,
        fallback_suffix: Option<&'static str>,
        priority: u8,
    ) -> Name {
        // `/` can't be part of an XML name, so the internal name never collides with named types
        let base = format!("{}/{}", parent.name, segment);
        let mut name = self.get_node_name(&base, false);
        let mut i = 1;
        while self.shared.anonymous_types.iter().any(|(n, _)| *n == name) {
            i += 1;
            name = self.get_node_name(&format!("{base}{i}"), false);
        }
        self.shared.anonymous_types.push((
            name.clone(),
            AnonymousType {
                parent: parent.clone(),
                segment: segment.to_string(),
                include_parent,
                fallback_suffix,
                priority,
            },
        ));
        name
    }

    /// The name of the given type relative to its parent, i.e. the segment for anonymous types,
    /// and the name itself for named ones.
    pub fn segment_name(&self, name: &Name) -> Name {
        match self.shared.anonymous_types.iter().find(|(n, _)| n == name) {
            Some((_, anonymous)) => self.get_node_name(&anonymous.segment, false),
            None => name.clone(),
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
        // unprefixed QNames are resolved using the default namespace
        match self.default_namespace {
            Some(NS_XSD) => Ok(LeafContent::Literal(literal_from_str(name).ok_or_else(
                || XsdError::UnsupportedType {
                    name: name.to_string(),
                    range: attr.range(),
                },
            )?)),
            Some(ns) => Ok(LeafContent::Named(Name::new(
                name,
                self.shared.namespaces.get_or_insert(ns),
            ))),
            None => Ok(LeafContent::Named(Name::new(name, Namespace::None))),
        }
    }

    pub fn target_namespace(&self) -> Namespace {
        self.target_namespace
    }
}

impl SchemaContext {
    /// The Rust identifier of the type with the given name.
    pub fn type_ident(&self, name: &Name) -> syn::Ident {
        match self.type_names.get(name) {
            Some(name) => format_ident!("{}", name),
            None => format_ident!("{}", type_ident_string(&name.name)),
        }
    }

    pub fn is_anonymous(&self, name: &Name) -> bool {
        self.type_names.contains_key(name)
    }

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
        "string" | "normalizedString" | "token" | "language" | "Name" | "NCName" | "NMTOKEN"
        | "ID" | "IDREF" | "anyURI" => LiteralType::String,
        "boolean" => LiteralType::Boolean,
        // TODO: add validation for negative/non-positive?
        "integer" | "long" | "negativeInteger" | "nonPositiveInteger" => LiteralType::Int64,
        "nonNegativeInteger" | "unsignedLong" => LiteralType::Uint64,
        // TODO: add validation for non zero?
        "positiveInteger" => LiteralType::Uint64,
        "int" => LiteralType::Int32,
        "unsignedInt" => LiteralType::Uint32,
        "short" => LiteralType::Int16,
        "unsignedShort" => LiteralType::Uint16,
        "byte" => LiteralType::Int8,
        "unsignedByte" => LiteralType::Uint8,
        "decimal" => LiteralType::Decimal,
        "float" => LiteralType::Float32,
        "double" => LiteralType::Float64,
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

fn type_ident_string(name: &str) -> String {
    escape_ident(&name.to_pascal_case()).to_string()
}

/// Assigns each anonymous type the shortest unique name. Names are build from the last segments
/// of the type's path (e.g. `Item`, `ItemsItem`, `OrderItemsItem`, ...). If the candidates of
/// multiple types collide, none of them get the name and all of them try the next longer
/// candidate. If a type's whole path is exhausted, its fallback suffix, or as a last resort a
/// number is appended.
fn resolve_type_names(
    roots: &HashMap<Name, Root>,
    anonymous_types: &[(Name, AnonymousType)],
) -> HashMap<Name, String> {
    let anonymous: HashMap<&Name, &AnonymousType> =
        anonymous_types.iter().map(|(n, a)| (n, a)).collect();
    let mut taken: HashSet<String> = roots
        .keys()
        .filter(|name| !anonymous.contains_key(name))
        .map(|name| type_ident_string(&name.name))
        .collect();

    // the path of segments from the named root type to each anonymous type
    let types: Vec<(&Name, Vec<&str>, &AnonymousType)> = anonymous_types
        .iter()
        .filter(|(name, _)| roots.contains_key(name))
        .map(|(name, anonymous_type)| {
            let mut path = vec![anonymous_type.segment.as_str()];
            let mut parent = &anonymous_type.parent;
            while let Some(anonymous_type) = anonymous.get(parent) {
                path.push(&anonymous_type.segment);
                parent = &anonymous_type.parent;
            }
            path.push(&parent.name);
            path.reverse();
            (name, path, anonymous_type)
        })
        .collect();

    let candidate = |path: &[&str], len: usize| {
        let name = path[path.len() - len..]
            .iter()
            .map(|segment| upper_first(segment))
            .collect::<String>();
        type_ident_string(&name)
    };

    let mut result = HashMap::with_capacity(types.len());
    let mut pending: Vec<(usize, usize)> = types
        .iter()
        .enumerate()
        .map(|(i, (_, path, anonymous_type))| {
            let min_len = if anonymous_type.include_parent { 2 } else { 1 };
            (i, min_len.min(path.len()))
        })
        .collect();
    let full_names = types
        .iter()
        .map(|(_, path, _)| candidate(path, path.len()))
        .collect::<Vec<_>>();
    // which type got which name
    let mut holders = HashMap::<String, usize>::new();
    let mut exhausted = Vec::new();
    while !pending.is_empty() {
        let candidates = pending
            .iter()
            .map(|(i, len)| candidate(&types[*i].1, *len))
            .collect::<Vec<_>>();
        let mut counts = HashMap::<&str, usize>::new();
        for candidate in &candidates {
            *counts.entry(candidate).or_default() += 1;
        }

        let mut next = Vec::new();
        for ((i, len), candidate) in pending.iter().zip(&candidates) {
            if counts[candidate.as_str()] == 1 && !taken.contains(candidate) {
                taken.insert(candidate.clone());
                holders.insert(candidate.clone(), *i);
                result.insert(types[*i].0.clone(), candidate.clone());
            } else if *len < types[*i].1.len() {
                next.push((*i, len + 1));
            } else if let Some(holder) = holders.get(candidate).copied().filter(|holder| {
                let holder_type = types[*holder].2;
                // same path (as Rust identifier), but a lower priority
                *candidate == full_names[*holder]
                    && types[*i].2.priority < holder_type.priority
                    && holder_type.fallback_suffix.is_some()
            }) {
                // Another type with the same path got the name first, e.g. a choice named after
                // its only element, while the type of that element still tried shorter names.
                // The one with the higher priority keeps it, the other one falls back to its
                // suffix instead of both being numbered.
                holders.insert(candidate.clone(), *i);
                result.insert(types[*i].0.clone(), candidate.clone());
                result.remove(types[holder].0);
                exhausted.push((holder, candidate.clone()));
            } else {
                exhausted.push((*i, candidate.clone()));
            }
        }
        pending = next;
    }

    // try the fallback suffixes first, which might also free up the candidate for others (only
    // the types with the highest priority among the ones with the same name keep it)
    let mut priorities = HashMap::<&str, u8>::new();
    for (i, candidate) in &exhausted {
        let priority = priorities.entry(candidate).or_insert(u8::MAX);
        *priority = (*priority).min(types[*i].2.priority);
    }
    let mut remaining = Vec::new();
    for (i, candidate) in &exhausted {
        let (i, candidate) = (*i, candidate.clone());
        let yields = taken.contains(&candidate) || types[i].2.priority > priorities[&*candidate];
        match types[i].2.fallback_suffix {
            Some(suffix) if yields && !taken.contains(&format!("{candidate}{suffix}")) => {
                let name = format!("{candidate}{suffix}");
                taken.insert(name.clone());
                result.insert(types[i].0.clone(), name);
            }
            _ => remaining.push((i, candidate)),
        }
    }

    let mut counts = HashMap::<String, usize>::new();
    for (_, candidate) in &remaining {
        *counts.entry(candidate.clone()).or_default() += 1;
    }
    for (i, candidate) in remaining {
        let name = if counts[&candidate] == 1 && !taken.contains(&candidate) {
            candidate
        } else {
            let mut n = 2;
            while taken.contains(&format!("{candidate}{n}")) {
                n += 1;
            }
            format!("{candidate}{n}")
        };
        taken.insert(name.clone());
        result.insert(types[i].0.clone(), name);
    }

    result
}

fn upper_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}
