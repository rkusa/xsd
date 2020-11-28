use super::{get_xml_name, ElementDefault, LeafContent, Name, Namespaces, State};
use crate::generator::escape_ident;
use inflector::Inflector;
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug, Clone)]
pub struct Attribute {
    pub name: Name,
    pub content: LeafContent,
    pub default: Option<String>,
    pub is_optional: bool,
}

impl Attribute {
    pub fn to_impl(&self, state: &mut State) -> TokenStream {
        let name_ident = escape_ident(&self.name.name.to_snake_case());
        let mut type_ident = self.content.to_impl(state);
        if self.is_optional {
            type_ident = quote! { Option<#type_ident> };
        }
        quote! { pub #name_ident: #type_ident, }
    }

    pub fn to_xml_impl(&self, element_default: &ElementDefault) -> TokenStream {
        let name_ident = escape_ident(&self.name.name.to_snake_case());
        let name_xml = get_xml_name(&self.name, element_default.qualified);
        let inner = match &self.content {
            LeafContent::Literal(literal) => literal.to_xml_impl(element_default),
            LeafContent::Named(_) => quote! { val.as_str() },
        };
        if self.is_optional {
            quote! {
                let val = self.#name_ident.as_ref().map(|val| {
                    #inner
                });
                if let Some(val) = val {
                    ctx.set_attr(#name_xml, val)
                }
            }
        } else {
            quote! {
                let val = &self.#name_ident;
                let val = #inner;
                ctx.set_attr(#name_xml, val);
            }
        }
    }

    pub fn from_xml_impl<'a>(
        &self,
        _element_default: &ElementDefault,
        _namespaces: &'a Namespaces<'a>,
    ) -> TokenStream {
        let name_ident = escape_ident(&self.name.name.to_snake_case());
        let name_xml = &self.name.name;
        let inner = self.content.from_str_impl();

        if self.is_optional {
            quote! {
                #name_ident: {
                    if let Some(val) = node.attribute(#name_xml) {
                        Some(#inner)
                    } else {
                        None
                    }
                },
            }
        } else {
            quote! {
                #name_ident: {
                    let val = node.try_attribute(#name_xml)?;
                    #inner
                },
            }
        }
    }
}
