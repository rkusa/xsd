use std::borrow::Cow;
use std::fs::read_to_string;
use std::io;
use std::ops::Range;
use std::{collections::HashMap, path::Path};

use super::context::{Context, NS_XSD};
use super::error::XsdError;
use super::node::Node;
use crate::ast::{get_xml_name, ElementDefault, Name, Namespace, Root};
use crate::utils::escape_ident;
use inflector::Inflector;
use proc_macro2::TokenStream;
use quote::{quote, TokenStreamExt};
use roxmltree::{Document, TextPos};

pub struct Schema {
    elements: HashMap<Name, Root>,
    target_namespace: Option<String>,
    qualified: bool,
}

impl Schema {
    pub fn parse_file(path: impl AsRef<Path>) -> Result<Self, SchemaError> {
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

        Schema::parse(&data, path)
    }

    pub fn parse(data: &str, path: impl AsRef<Path>) -> Result<Self, SchemaError> {
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
        Schema::parse_schema(&root_node, base_path, None).map_err(|err| match err {
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
        })
    }

    pub fn parse_schema(
        root: &Node<'_, '_>,
        base_path: &Path,
        target_namespace: Option<&str>,
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
        let qualified = root
            .attribute("elementFormDefault")
            .map(|a| a.value())
            .as_deref()
            == Some("qualified");
        let mut ctx = Context::new(&root, target_namespace.as_deref());

        for child in root.children().namespace(NS_XSD).iter() {
            // TODO: prevent circular includes
            if child.name() == "include" {
                // TODO: make sure that it is a relative path
                let location: &str = &child.try_attribute("schemaLocation")?.value();
                let mut path = base_path.to_path_buf();
                path.push(location);

                // merge imports
                let import = Schema::parse_file(path)?;
                for (name, root) in import.elements {
                    ctx.add_root(name, root);
                }

                continue;
            }

            let name = Name::new(
                child.try_attribute("name")?.value(),
                // Root elements and types are qualified to the target namespace if there is one
                if target_namespace.is_some() {
                    Namespace::Target
                } else {
                    Namespace::None
                },
            );

            let root = crate::xsd::parse::root::parse(child, &name, &ctx)?;
            ctx.add_root(name, root);
        }

        Ok(Schema {
            elements: ctx.take_roots(),
            target_namespace: target_namespace.map(String::from),
            qualified,
        })
    }

    pub fn elements(&self) -> impl Iterator<Item = (&Name, &Root)> {
        self.elements.iter()
    }

    pub fn element_names(&self) -> impl Iterator<Item = &Name> {
        self.elements.keys()
    }

    pub fn target_namespace(&self) -> Option<&str> {
        self.target_namespace.as_deref()
    }

    pub fn qualified(&self) -> bool {
        self.qualified
    }

    pub fn generate_element(&self, name: &Name) -> Result<TokenStream, SchemaError> {
        let el = self
            .elements
            .get(name)
            .ok_or_else(|| SchemaError::NotFound {
                name: name.name.clone(),
            })?;

        // TODO: derive from schema
        let namespaces = HashMap::new();
        let element_default = ElementDefault {
            target_namespace: self.target_namespace().map(|tn| tn.to_string()),
            qualified: self.qualified(),
        };
        let mut result = TokenStream::new();
        let mut state = ();

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

        let to_xml = el.to_xml_impl(&element_default);

        let name_xml = get_xml_name(&name, element_default.qualified);
        let mut element_ns = Vec::new();
        if let Some(tn) = self.target_namespace() {
            if self.qualified() {
                element_ns.push(quote! { .set_default_ns(#tn) });
            } else {
                element_ns.push(quote! { .set_ns("tn", #tn) });
            }
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
        let namespace_xml = name.namespace.to_quote(&element_default);
        let from_xml = el.from_xml_impl(&name_ident, &element_default, &namespaces);

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

        let lookahead = el.lookahead_impl(&element_default);

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
