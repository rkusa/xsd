use std::borrow::Cow;
use std::fs::read_to_string;
use std::io;
use std::{collections::HashMap, path::Path};

use roxmltree::{Document, TextPos};

use super::context::{Context, NS_XSD};
use super::error::XsdError;
use super::node::Node;
use crate::ast::{Name, Namespace, Root};

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

    fn parse_schema(
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

    pub fn target_namespace(&self) -> Option<&str> {
        self.target_namespace.as_deref()
    }

    pub fn qualified(&self) -> bool {
        self.qualified
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
}

#[derive(Debug, thiserror::Error)]
enum ParseError {
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
