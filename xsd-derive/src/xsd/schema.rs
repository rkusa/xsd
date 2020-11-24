use std::{collections::HashMap, path::Path};

use roxmltree::{Document, TextPos};

use super::context::{Context, NS_XSD};
use super::error::{Error, XsdError};
use super::node::Node;
use crate::types::{ElementDefinition, Name, Namespace};

pub struct Schema {
    elements: HashMap<Name, ElementDefinition>,
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

        let mut ctx = Context::new(&root, target_namespace);

        for child in root.children().namespace(NS_XSD).iter() {
            if child.name() == "import" {
                unimplemented!("import");
                // continue;
            }

            let name = Name::new(child.try_attribute("name")?.value(), Namespace::Target);

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
        for (name, definition) in ctx.elements() {
            elements.insert(name.clone(), definition.try_get(&ctx)?.clone());
        }

        Ok(Schema { elements })
    }

    pub fn elements(&self) -> impl Iterator<Item = (&Name, &ElementDefinition)> {
        self.elements.iter()
    }
}
