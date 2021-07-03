use std::borrow::Cow;
use std::collections::HashSet;
use std::fs::read_to_string;
use std::io;
use std::ops::Range;
use std::{collections::HashMap, path::Path};

use super::context::{Context, SchemaContext, SharedContext, NS_XSD};
use super::error::XsdError;
use super::node::Node;
use crate::ast::{Name, Namespace, Root};
use crate::utils::escape_ident;
use inflector::Inflector;
use proc_macro2::TokenStream;
use quote::{quote, TokenStreamExt};
use roxmltree::{Document, TextPos};

pub struct Schema {
    pub(crate) elements: HashMap<Name, Root>,
    pub(crate) dependencies: HashMap<Name, HashSet<Name>>,
    pub(crate) context: SchemaContext,
}

impl Schema {
    pub fn parse_file(path: impl AsRef<Path>) -> Result<Self, SchemaError> {
        Self::parse_file_with_context(path, None)
    }

    fn parse_file_with_context(
        path: impl AsRef<Path>,
        shared: Option<SharedContext>,
    ) -> Result<Self, SchemaError> {
        let path = path.as_ref();

        let data = match read_to_string(&path) {
            Ok(data) => data,
            Err(err) => {
                return Err(SchemaError::Open {
                    err,
                    file: path.to_string_lossy().to_string(),
                })
            }
        };

        Schema::parse_with_context(&data, path, shared)
    }

    pub fn parse(data: &str, path: impl AsRef<Path>) -> Result<Self, SchemaError> {
        Self::parse_with_context(data, path, None)
    }

    fn parse_with_context(
        data: &str,
        path: impl AsRef<Path>,
        shared: Option<SharedContext>,
    ) -> Result<Self, SchemaError> {
        let path = path.as_ref().to_path_buf();
        let doc = match Document::parse(data) {
            Ok(doc) => doc,
            Err(err) => {
                let pos = err.pos();
                return Err(SchemaError::Xsd {
                    file: path.to_string_lossy().to_string(),
                    row: pos.row,
                    col: pos.col,
                    err: Box::new(XsdError::Xml(err)),
                });
            }
        };

        let base_path = path.parent().unwrap_or_else(|| path.as_path());
        let root_node = doc.root_element().into();
        Schema::parse_schema_with_context(&root_node, base_path, None, shared).map_err(|err| {
            match err {
                ParseError::Schema(err) => err,
                ParseError::Xsd(err) => {
                    let pos = err
                        .range()
                        .map(|range| doc.text_pos_at(range.start))
                        .unwrap_or_else(|| TextPos { row: 0, col: 0 });
                    SchemaError::Xsd {
                        file: path.to_string_lossy().to_string(),
                        row: pos.row,
                        col: pos.col,
                        err: Box::new(err),
                    }
                }
            }
        })
    }

    pub fn parse_schema(
        root: &Node<'_, '_>,
        base_path: &Path,
        target_namespace: Option<&str>,
    ) -> Result<Self, ParseError> {
        Self::parse_schema_with_context(root, base_path, target_namespace, None)
    }

    fn parse_schema_with_context(
        root: &Node<'_, '_>,
        base_path: &Path,
        target_namespace: Option<&str>,
        shared: Option<SharedContext>,
    ) -> Result<Self, ParseError> {
        if root.namespace().as_deref() != Some(NS_XSD) || root.name() != "schema" {
            return Err(XsdError::UnsupportedElement {
                name: root.name().to_string(),
                range: root.range(),
            }
            .into());
        }

        let target_namespace = target_namespace
            .map(Cow::Borrowed)
            .or_else(|| root.attribute("targetNamespace").map(|a| a.value()));

        let mut ctx = Context::new(
            &root,
            target_namespace.as_deref(),
            shared.unwrap_or_default(),
        );

        for child in root.children().namespace(NS_XSD).iter() {
            // TODO: prevent circular includes
            if child.name() == "include" || child.name() == "import" {
                // TODO: make sure that it is a relative path
                let location: &str = &child.try_attribute("schemaLocation")?.value();
                let mut path = base_path.to_path_buf();
                path.push(location);

                // merge imports
                let mut schema = Schema::parse_file_with_context(path, Some(ctx.take_shared()))?;
                for (name, root) in std::mem::take(&mut schema.elements) {
                    ctx.add_root(name, root);
                }
                ctx.set_shared(schema.into_shared());

                continue;
            }

            let name = Name::new(
                child.try_attribute("name")?.value(),
                // Root elements and types are qualified to the target namespace if there is one
                ctx.target_namespace(),
            );

            let root = crate::xsd::parse::root::parse(child, &name, &mut ctx)?;
            ctx.add_root(name, root);
        }

        Ok(ctx.into_schema())
    }

    fn into_shared(self) -> SharedContext {
        SharedContext {
            namespaces: self.context.namespaces,
            dependencies: self.dependencies,
        }
    }

    pub fn elements(&self) -> impl Iterator<Item = (&Name, &Root)> {
        self.elements.iter()
    }

    pub fn generate_all(&self) -> Result<TokenStream, SchemaError> {
        let state = ();
        let mut result = TokenStream::new();

        for name in self.elements.keys() {
            result.append_all(self.generate_element(name, state)?);
        }

        Ok(result)
    }

    pub fn generate_element_and_dependencies<'a>(
        &'a self,
        name: &'a Name,
        already_generated: &mut HashSet<&'a Name>,
    ) -> Result<TokenStream, SchemaError> {
        let state = ();
        let mut result = TokenStream::new();

        let mut names = HashSet::with_capacity(1);
        let mut new_names = HashSet::with_capacity(1);
        new_names.insert(name);

        loop {
            if new_names.is_empty() {
                break;
            }

            let mut next_names = HashSet::new();
            for name in &new_names {
                if let Some(dependends) = self.dependencies.get(name) {
                    for d in dependends {
                        if !names.contains(d) && !new_names.contains(d) {
                            next_names.insert(d);
                        }
                    }
                }
            }

            names.extend(new_names);
            new_names = next_names;
        }

        for name in names {
            if already_generated.contains(name) {
                continue;
            }
            result.append_all(self.generate_element(name, state)?);
            already_generated.insert(name);
        }

        Ok(result)
    }

    fn generate_element(&self, name: &Name, mut state: ()) -> Result<TokenStream, SchemaError> {
        let el = self
            .elements
            .get(name)
            .ok_or_else(|| SchemaError::NotFound {
                name: name.name.clone(),
            })?;

        let mut result = TokenStream::new();

        // TODO: handle duplicates with different prefixes
        let name_ident = escape_ident(&name.name.to_pascal_case());
        let kind = if el.is_enum() {
            quote!(enum)
        } else {
            quote!(struct)
        };
        let declaration = &el.to_declaration(&name_ident, &mut state);
        let docs = el
            .docs()
            .map(|docs| quote! { #[doc = #docs] })
            .unwrap_or_else(TokenStream::new);

        result.append_all(quote! {
            #docs
            #[derive(Debug, Clone, PartialEq)]
            pub #kind #name_ident#declaration
        });

        let to_xml = el.to_xml_impl(&self.context);

        let name_xml = self.context.get_xml_element_name(&name);
        let mut element_ns = Vec::new();
        if self.context.is_qualified {
            if let Namespace::Id(id) = self.context.target_namespace {
                let ns = self.context.namespaces.get_by_id(id);
                let namespace = &ns.namespace;
                element_ns.push(quote! { .set_default_ns(#namespace) });
            }
        }
        for (id, ns) in self.context.namespaces.iter() {
            if self.context.is_qualified && Namespace::Id(id) == self.context.target_namespace {
                continue;
            }
            let prefix = &ns.prefix;
            let namespace = &ns.namespace;
            element_ns.push(quote! { .set_ns(#prefix, #namespace) });
        }

        result.append_all(quote! {
            impl #name_ident {
                pub fn to_xml(&self) -> Result<Vec<u8>, ::xsd::xml::writer::Error> {
                    use ::xsd::xml::writer::events::XmlEvent;

                    let mut body = Vec::new();
                    let mut writer = ::xsd::xml::writer::EmitterConfig::new()
                        .perform_indent(true)
                        .create_writer(&mut body);

                    writer.write(XmlEvent::StartDocument {
                        version: ::xsd::xml::common::XmlVersion::Version10,
                        encoding: Some("UTF-8"),
                        standalone: None,
                    })?;
                    let mut ctx = ::xsd::Context::new(#name_xml);
                    self.to_xml_writer(ctx, &mut writer)?;

                    Ok(body)
                }

                fn to_xml_writer<'a, 'b, W: ::std::io::Write>(
                    &'a self,
                    mut ctx: ::xsd::Context<'a, 'b>,
                    writer: &mut ::xsd::xml::writer::EventWriter<W>,
                ) -> Result<(), ::xsd::xml::writer::Error> {
                    use ::xsd::xml::writer::events::XmlEvent;

                    #(ctx#element_ns;)*
                    #to_xml

                    Ok(())
                }
            }
        });

        let name_xml = &name.name;
        let namespace_xml = self.context.quote_xml_namespace(&name);
        let from_xml = el.from_xml_impl(&name_ident, &self.context);

        result.append_all(quote! {
            impl #name_ident {
                pub fn from_xml(input: impl AsRef<str>) -> Result<Self, ::xsd::decode::FromXmlError> {
                    let doc = ::xsd::decode::decode(input.as_ref())?;
                    let node = doc.try_child(#name_xml, #namespace_xml)?;
                    Self::from_xml_node(&node)
                }

                fn from_xml_node(node: &::xsd::decode::Node) -> Result<Self, ::xsd::decode::FromXmlError> {
                    Ok(#from_xml)
                }
            }
        });

        let lookahead = el.lookahead_impl(&self.context);

        result.append_all(quote! {
            impl #name_ident {
                fn lookahead(node: &::xsd::decode::Node) -> bool {
                    #lookahead
                }
            }
        });

        // eprintln!("{}", result.to_string());
        Ok(result)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SchemaError {
    #[error("Error on line {row} (offset {col} in {file}): {err}")]
    Xsd {
        #[source]
        err: Box<XsdError>,
        file: String,
        col: u32,
        row: u32,
    },
    #[error("Failed to load XSD include {file}: {err}")]
    Open {
        #[source]
        err: io::Error,
        file: String,
    },
    #[error("Element {name} not found in schema")]
    NotFound { name: String },
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("{0}")]
    Schema(#[from] SchemaError),
    #[error("{0}")]
    Xsd(#[from] XsdError),
}

impl From<super::node::NodeError> for ParseError {
    fn from(err: super::node::NodeError) -> Self {
        ParseError::Xsd(err.into())
    }
}

impl ParseError {
    pub fn range(&self) -> Option<&Range<usize>> {
        match self {
            ParseError::Schema(_) => None,
            ParseError::Xsd(err) => err.range(),
        }
    }
}
