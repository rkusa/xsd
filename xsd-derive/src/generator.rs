use std::collections::HashMap;
use std::{fs::read_to_string, path::Path};

use crate::ast::{
    get_xml_name, ElementContent, ElementDefault, FromXmlImpl, Kind, ToImpl, ToXmlImpl,
};
use crate::error::GeneratorError;
use crate::xsd::Schema;
use inflector::Inflector;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, TokenStreamExt};

pub fn generate(
    item: &syn::ItemMod,
    path: impl AsRef<Path>,
) -> Result<TokenStream, GeneratorError> {
    let path = path.as_ref();
    let base_path = path.parent().unwrap_or(path);

    let data = match read_to_string(&path) {
        Ok(data) => data,
        Err(err) => {
            return Err(GeneratorError::Open {
                err,
                file: path.to_string_lossy().to_string(),
            })
        }
    };

    let schema = Schema::parse(&data, base_path)?;
    // TODO: derive from schema
    let namespaces = HashMap::new();
    let element_default = ElementDefault {
        target_namespace: schema.target_namespace().map(|tn| tn.to_string()),
        qualified: schema.qualified(),
    };
    let mut structs = TokenStream::new();

    let mut state = ();
    for (name, el) in schema.elements() {
        // println!("{:#?}", el);

        // TODO: handle duplicates with different prefixes
        let name_ident = escape_ident(&name.name.to_pascal_case());
        let struct_body = &el.content.to_impl(&mut state);

        structs.append_all(quote! {
            #[derive(Debug, Clone, PartialEq)]
            pub struct #name_ident#struct_body
        });

        let to_xml = match &el.content {
            ElementContent::Literal(literal) => {
                let inner = literal.to_xml_impl(&element_default);
                quote! {
                    let val = &self.0;
                    #inner
                }
            }
            content => content.to_xml_impl(&element_default),
        };

        let name_xml = get_xml_name(&name, element_default.qualified);
        let mut element_ns = Vec::new();
        if let Some(tn) = schema.target_namespace() {
            if schema.qualified() {
                element_ns.push(quote! { .default_ns(#tn) });
            } else {
                element_ns.push(quote! { .ns("tn", #tn) });
            }
        }

        if el.kind == Kind::Root {
            structs.append_all(quote! {
                impl #name_ident {
                    pub fn to_xml(&self) -> Result<Vec<u8>, ::xml::writer::Error> {
                        use ::xml::writer::events::XmlEvent;

                        let mut body = Vec::new();
                        let mut writer = ::xml::writer::EmitterConfig::new()
                            .perform_indent(true)
                            .create_writer(&mut body);

                        writer.write(XmlEvent::StartDocument {
                            version: ::xml::common::XmlVersion::Version10,
                            encoding: Some("UTF-8"),
                            standalone: None,
                        })?;
                        self.to_xml_writer(&mut writer)?;

                        Ok(body)
                    }
                }
            });
        }

        match el.kind {
            Kind::Root | Kind::Child => {
                structs.append_all(quote! {
                    impl #name_ident {
                        fn to_xml_writer<W: ::std::io::Write>(
                            &self,
                            writer: &mut ::xml::writer::EventWriter<W>,
                        ) -> Result<(), ::xml::writer::Error> {
                            use ::xml::writer::events::XmlEvent;

                            writer.write(XmlEvent::start_element(#name_xml)
                                #(#element_ns)*)?;
                            #to_xml
                            writer.write(XmlEvent::end_element())?;

                            Ok(())
                        }
                    }
                });
            }
            Kind::Virtual => {
                structs.append_all(quote! {
                    impl #name_ident {
                        fn to_xml_writer<W: ::std::io::Write>(
                            &self,
                            writer: &mut ::xml::writer::EventWriter<W>,
                        ) -> Result<(), ::xml::writer::Error> {
                            use ::xml::writer::events::XmlEvent;

                            #to_xml

                            Ok(())
                        }
                    }
                });
            }
        }

        let name_xml = &name.name;
        let namespace_xml = name.namespace.from_xml_impl(&element_default, &namespaces);
        let from_xml = el.from_xml_impl(&element_default, &namespaces);

        if el.kind == Kind::Root {
            structs.append_all(quote! {
                impl #name_ident {
                    pub fn from_xml(input: impl AsRef<str>) -> Result<Self, ::xsd::decode::FromXmlError> {
                        let doc = ::xsd::decode::decode(input.as_ref())?;
                        let node = doc.try_child(#name_xml, #namespace_xml)?;
                        Self::from_xml_node(&node)
                    }
                }
            });
        }

        structs.append_all(quote! {
            impl #name_ident {
                fn from_xml_node(node: &::xsd::decode::Node) -> Result<Self, ::xsd::decode::FromXmlError> {
                    Ok(#name_ident#from_xml)
                }
            }
        });
    }

    let attrs = &item.attrs;
    let vis = &item.vis;
    let ident = &item.ident;
    let items = item
        .content
        .as_ref()
        .map(|(_, items)| items.clone())
        .unwrap_or_default();

    Ok(quote! {
        #(#attrs)*
        #vis mod #ident {
            #(#items
            )*

            #structs
        }
    })
}

pub fn escape_ident(name: &str) -> syn::Ident {
    match name {
        " break" | "const" | "continue" | "crate" | "else" | "enum" | "extern" | "false" | "fn"
        | "for" | "if" | "impl" | "in" | "let" | "loop" | "match" | "mod" | "move" | "mut"
        | "pub" | "ref" | "return" | "self" | "Self" | "static" | "struct" | "super" | "trait"
        | "true" | "type" | "unsafe" | "use" | "where" | "while" | "async" | "await" | "dyn" => {
            format_ident!("r#{}", name)
        }
        _ => format_ident!("{}", name),
    }
}
