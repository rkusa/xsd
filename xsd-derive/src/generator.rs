use std::collections::HashMap;
use std::{fs::read_to_string, path::Path};

use crate::ast::{get_xml_name, ElementDefault};
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
        // eprintln!("{:#?} {:#?}", name, el);

        // TODO: handle duplicates with different prefixes
        let name_ident = escape_ident(&name.name.to_pascal_case());
        let kind = if el.is_enum() {
            quote!(enum)
        } else {
            quote!(struct)
        };
        let declaration = &el.to_declaration(&name_ident, &mut state);

        structs.append_all(quote! {
            #[derive(Debug, Clone, PartialEq)]
            pub #kind #name_ident#declaration
        });

        let to_xml = el.to_xml_impl(&element_default);

        let name_xml = get_xml_name(&name, element_default.qualified);
        let mut element_ns = Vec::new();
        if let Some(tn) = schema.target_namespace() {
            if schema.qualified() {
                element_ns.push(quote! { .set_default_ns(#tn) });
            } else {
                element_ns.push(quote! { .set_ns("tn", #tn) });
            }
        }

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
                    let mut ctx = ::xsd::Context::new(#name_xml);
                    self.to_xml_writer(ctx, &mut writer)?;

                    Ok(body)
                }

                fn to_xml_writer<'a, 'b, W: ::std::io::Write>(
                    &'a self,
                    mut ctx: ::xsd::Context<'a, 'b>,
                    writer: &mut ::xml::writer::EventWriter<W>,
                ) -> Result<(), ::xml::writer::Error> {
                    use ::xml::writer::events::XmlEvent;

                    #(ctx#element_ns;)*
                    #to_xml

                    Ok(())
                }
            }
        });

        let name_xml = &name.name;
        let namespace_xml = name.namespace.from_xml_impl(&element_default, &namespaces);
        let from_xml = el.from_xml_impl(&name_ident, &element_default, &namespaces);

        structs.append_all(quote! {
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
    }

    let attrs = &item.attrs;
    let vis = &item.vis;
    let ident = &item.ident;
    let items = item
        .content
        .as_ref()
        .map(|(_, items)| items.clone())
        .unwrap_or_default();

    let result = quote! {
        #(#attrs)*
        #vis mod #ident {
            #(#items
            )*

            #structs
        }
    };

    // eprintln!("{}", result.to_string());
    Ok(result)
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
