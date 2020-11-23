mod error;

use std::collections::HashMap;
use std::{fs::read_to_string, path::Path};

use crate::types::{ElementContent, ElementDefault, FromXmlImpl, LiteralType, ToImpl, ToXmlImpl};
use crate::xsd::Schema;
use error::GeneratorError;
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
    let element_default = ElementDefault::Unqualified("TODO".to_string());
    let mut structs = TokenStream::new();

    let mut state = ();
    for (name, el) in schema.elements() {
        // TODO: handle duplicates with different prefixes
        let name_ident = escape_ident(&name.name.to_pascal_case());
        let name_xml = element_default.get_xml_name(&name);

        let struct_body = el.content.to_impl(&mut state);
        let to_xml = el.to_xml_impl(&element_default);
        let from_xml = el.from_xml_impl(&element_default, &namespaces);

        structs.append_all(quote! {
            #[derive(Debug, Clone, PartialEq)]
            pub struct #name_ident#struct_body

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

                    writer.write(XmlEvent::start_element(#name_xml))?;
                    #to_xml
                    writer.write(XmlEvent::end_element())?;

                    Ok(body)
                }

                pub fn from_xml(input: impl AsRef<str>) -> Result<Self, ::xsd::decode::FromXmlError> {
                    let doc = ::xsd::decode::decode(input.as_ref())?;
                    // TODO: namespace
                    let node = doc.try_child(#name_xml, None)?;
                    Ok(#name_ident#from_xml)
                }
            }
        });
    }

    let attrs = &item.attrs;
    let vis = &item.vis;
    let ident = &item.ident;

    // TODO: keep existing content?

    Ok(quote! {
        #(#attrs)*
        #vis mod #ident {
            #structs
        }
    })
}

fn escape_ident(name: &str) -> syn::Ident {
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
