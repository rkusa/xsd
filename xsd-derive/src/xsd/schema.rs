use std::{collections::HashMap, path::Path};

use roxmltree::{Document, TextPos};

use super::context::{Context, NS_XSD};
use super::error::{Error, XsdError};
use super::node::Node;
use crate::ast::{Name, Namespace, Root};

pub struct Schema {
    elements: HashMap<Name, Root>,
    target_namespace: Option<String>,
    qualified: bool,
}

impl Schema {
    pub fn parse(data: &str, path: impl AsRef<Path>) -> Result<Self, Error> {
        let path = path.as_ref().to_path_buf();
        let doc = match Document::parse(&data) {
            Ok(doc) => doc,
            Err(err) => {
                let pos = err.pos();
                return Err(Error::Xsd {
                    file: path.to_string_lossy().to_string(),
                    row: pos.row,
                    col: pos.col,
                    err: Box::new(XsdError::Xml(err)),
                });
            }
        };

        let base_path = path.parent().unwrap_or_else(|| path.as_path());
        Schema::parse_schema(doc.root_element().into(), base_path, None).map_err(|err| {
            let pos = err
                .range()
                .map(|range| doc.text_pos_at(range.start))
                .unwrap_or_else(|| TextPos { row: 0, col: 0 });
            Error::Xsd {
                file: path.to_string_lossy().to_string(),
                row: pos.row,
                col: pos.col,
                err: Box::new(err),
            }
        })
    }

    fn parse_schema(
        root: Node<'_, '_>,
        _base_path: &Path,
        target_namespace: Option<&str>,
    ) -> Result<Self, XsdError> {
        if root.namespace().as_deref() != Some(NS_XSD) || root.name() != "schema" {
            return Err(XsdError::UnsupportedElement {
                name: root.name().to_string(),
                parent: "".to_string(),
                range: root.range(),
            });
        }

        let target_namespace = target_namespace.map(|tn| tn.to_string()).or_else(|| {
            root.attribute("targetNamespace")
                .map(|a| a.value().to_string())
        });
        let qualified = root
            .attribute("elementFormDefault")
            .map(|a| a.value())
            .as_deref()
            == Some("qualified");
        let mut ctx = Context::new(&root, target_namespace.as_deref());

        for child in root.children().namespace(NS_XSD).iter() {
            if child.name() == "import" {
                unimplemented!("import");
                // continue;
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

            match child.name() {
                "element" => {
                    ctx.add_element(name, child);
                }
                "simpleType" => {
                    ctx.add_simple_type(name, child);
                }
                "complexType" => {
                    ctx.add_complex_type(name, child);
                }
                child_name => {
                    return Err(XsdError::UnsupportedElement {
                        name: child_name.to_string(),
                        parent: root.name().to_string(),
                        range: child.range(),
                    })
                }
            }
        }

        let mut elements = HashMap::new();
        loop {
            let mut has_more = false;
            for (name, definition) in ctx.remove_elements() {
                has_more = true;
                elements.insert(name.clone(), definition.try_get(&mut ctx)?.clone());
            }
            if !has_more {
                break;
            }
        }

        Ok(Schema {
            elements,
            target_namespace,
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
