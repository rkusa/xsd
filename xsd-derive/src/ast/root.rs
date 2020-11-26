use std::collections::HashMap;

use super::{ElementDefault, ElementDefinition, Leaf, LeafDefinition, Name, Namespaces, State};
use inflector::Inflector;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

#[derive(Debug, Clone)]
pub enum Root {
    Leaf(LeafDefinition),
    Enum(Vec<Name>),
    Element(ElementDefinition),
    Choice(Vec<Leaf>),
}

impl Root {
    pub fn is_enum(&self) -> bool {
        matches!(self, Root::Enum(_) | Root::Choice(_))
    }

    pub fn to_declaration(&self, root_name: &Ident, state: &mut State) -> TokenStream {
        match self {
            Root::Leaf(def) => {
                let inner = def.to_impl(state);
                quote! {
                    (pub #inner);
                }
            }
            Root::Enum(names) => {
                let names = escape_enum_names(names.clone());
                let variants = names.keys().map(|k| format_ident!("{}", k));
                let from_str_variants = names.iter().map(|(variant, name)| {
                    let name_xml = &name.name;
                    let variant = format_ident!("{}", variant);
                    quote! {
                        #name_xml => #root_name::#variant
                    }
                });
                let as_str_variants = names.iter().map(|(variant, name)| {
                    let name_xml = &name.name;
                    let variant = format_ident!("{}", variant);
                    quote! {
                        #root_name::#variant => #name_xml
                    }
                });

                quote! {
                    {
                        #(#variants,)*
                    }

                    impl ::std::str::FromStr for #root_name {
                        type Err = ::xsd::decode::FromXmlError;

                        fn from_str(s: &str) -> Result<Self, Self::Err> {
                            Ok(match s {
                                #(#from_str_variants,)*
                                _ => return Err(::xsd::decode::FromXmlError::InvalidVariant {
                                    name: s.to_string(),
                                })
                            })
                        }
                    }

                    impl #root_name {
                        pub fn as_str(&self) -> &str {
                            match self {
                                #(#as_str_variants,)*
                            }
                        }
                    }
                }
            }
            Root::Element(def) => {
                let inner = def.to_impl(state);
                quote! {
                    {
                        #inner
                    }
                }
            }
            Root::Choice(variants) => {
                let names = escape_enum_names(variants.iter().map(|v| v.name.clone()).collect());
                let variants = names.keys().map(|k| format_ident!("{}", k));

                quote! {
                    {
                        #(#variants(#variants),)*
                    }
                }
            }
        }
    }

    pub fn to_xml_impl(&self, element_default: &ElementDefault) -> TokenStream {
        match self {
            Root::Leaf(def) => {
                let inner = def.to_xml_impl(element_default);
                quote! {
                    let val = &self.0;
                    #inner
                }
            }
            Root::Enum(_) => {
                quote! {
                    writer.write(start)?;
                    let val = self.as_str();
                    writer.write(XmlEvent::characters(&val))?;
                }
            }
            Root::Element(def) => def.to_xml_impl(element_default),
            Root::Choice(variants) => {
                let names = escape_enum_names(variants.iter().map(|v| v.name.clone()).collect());
                let variants = names.into_iter().map(|(variant, name)| {
                    let ident = format_ident!("{}", variant);
                    let name_xml = &name.name;
                    quote! {
                        Self::#ident(val) => {
                            let start = XmlEvent::start_element(#name_xml);
                            val.to_xml_writer(start, writer)?
                        }
                    }
                });

                quote! {
                    match self {
                        #(#variants,)*
                    }
                }
            }
        }
    }

    pub fn from_xml_impl<'a>(
        &self,
        name: &Ident,
        element_default: &ElementDefault,
        namespaces: &'a Namespaces<'a>,
    ) -> TokenStream {
        match self {
            Root::Leaf(def) => {
                let inner = def.from_xml_impl(element_default, namespaces);
                quote! {
                    #name(#inner)
                }
            }
            Root::Enum(_) => {
                quote! {
                    {
                        let val = node.text()?;
                       ::std::str::FromStr::from_str(val)?
                    }
                }
            }
            Root::Element(def) => {
                let inner = def.from_xml_impl(element_default, namespaces);
                quote! {
                    #name#inner
                }
            }
            Root::Choice(variants) => {
                let names = escape_enum_names(variants.iter().map(|v| v.name.clone()).collect());
                let variants = names.into_iter().map(|(variant, name)| {
                    let ident = format_ident!("{}", variant);
                    let name_xml = &name.name;
                    let namespace_xml = name.namespace.from_xml_impl(&element_default, &namespaces);
                    quote! {
                        if let Some(node) = node.child(#name_xml, #namespace_xml) {
                             Self::#ident(#ident::from_xml_node(&node)?)
                        }
                    }
                });

                quote! {
                    #(#variants else )* {
                        return Err(::xsd::decode::FromXmlError::MissingVariant)
                    }
                }
            }
        }
    }
}

fn escape_enum_names(names: Vec<Name>) -> HashMap<String, Name> {
    let mut unknown_count = 0;
    let mut enum_names = HashMap::with_capacity(names.len());

    for name in names.into_iter() {
        let mut variant_name = name
            .name
            .chars()
            .filter_map(|c| match c {
                '_' | '-' => Some('_'),
                c => {
                    if c.is_alphanumeric() {
                        Some(c)
                    } else {
                        None
                    }
                }
            })
            .collect::<String>()
            .to_pascal_case();
        if !variant_name.is_empty()
            && variant_name
                .chars()
                .next()
                .map(|c| !c.is_alphabetic())
                .unwrap_or(false)
        {
            variant_name = format!("V{}", variant_name);
        }

        loop {
            if variant_name.is_empty() || enum_names.contains_key(&variant_name) {
                unknown_count += 1;
                variant_name = format!("Variant{}", unknown_count);
            } else {
                break;
            }
        }

        enum_names.insert(variant_name, name);
    }

    enum_names
}
