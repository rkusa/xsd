use super::{LeafContent, Name};
use crate::utils::escape_ident;
use crate::xsd::context::SchemaContext;
use inflector::Inflector;
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug, Clone)]
pub struct Attribute {
    pub name: Name,
    pub content: LeafContent,
    pub default: Option<String>,
    pub is_optional: bool,
    pub docs: Option<String>,
}

impl Attribute {
    pub fn to_impl(&self, ctx: &SchemaContext) -> TokenStream {
        let name_ident = escape_ident(&self.name.name.to_snake_case());
        let mut type_ident = self.content.to_impl(ctx);
        if self.is_optional {
            type_ident = quote! { Option<#type_ident> };
        }
        let docs = self
            .docs
            .as_ref()
            .filter(|docs| !docs.is_empty())
            .map(|docs| quote! { #[doc = #docs] })
            .unwrap_or_default();
        quote! {
            #docs
            pub #name_ident: #type_ident,
        }
    }

    pub fn to_xml_impl(&self, ctx: &SchemaContext) -> TokenStream {
        let name_ident = escape_ident(&self.name.name.to_snake_case());
        let name_xml = ctx.get_xml_element_name(&self.name);
        let inner = match &self.content {
            LeafContent::Literal(literal) => literal.to_xml_impl(),
            LeafContent::Named(_) => quote! { val.to_string() },
            LeafContent::Fixed(fixed) => quote! { #fixed },
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

    pub fn from_xml_impl(&self) -> TokenStream {
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
