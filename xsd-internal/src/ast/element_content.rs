use crate::utils::escape_ident;
use crate::xsd::context::SchemaContext;

use super::{Leaf, LeafDefinition, Name};
use inflector::Inflector;
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug, Clone)]
pub enum ElementContent {
    Leaf(Name, LeafDefinition),
    Leaves(Vec<Leaf>),
}

impl ElementContent {
    pub fn to_impl(&self, ctx: &SchemaContext) -> TokenStream {
        match self {
            ElementContent::Leaf(name, definition) => {
                let name_ident = escape_ident(&name.name.to_snake_case());
                let docs = definition
                    .docs
                    .as_ref()
                    .filter(|docs| !docs.is_empty())
                    .map(|docs| quote! { #[doc = #docs] })
                    .unwrap_or_else(TokenStream::new);
                let inner = definition.to_impl(ctx);
                quote! {
                    #docs
                    pub #name_ident: #inner,
                }
            }
            ElementContent::Leaves(leaves) => {
                let properties = leaves.iter().map(|el| el.to_impl(ctx)).collect::<Vec<_>>();
                quote!(#(#properties,)*)
            }
        }
    }

    pub fn to_xml_impl(&self, ctx: &SchemaContext) -> TokenStream {
        match &self {
            ElementContent::Leaf(name, definition) => {
                let name_ident = escape_ident(&name.name.to_snake_case());
                let inner = definition.to_xml_impl(ctx);
                quote! {
                    let val = &self.#name_ident;
                    #inner;
                }
            }
            ElementContent::Leaves(leaves) => {
                let properties = leaves
                    .iter()
                    .map(|el| el.to_xml_impl(ctx))
                    .collect::<Vec<_>>();
                quote! {
                    #(#properties)*
                }
            }
        }
    }

    pub fn from_xml_impl(&self, ctx: &SchemaContext) -> TokenStream {
        match &self {
            ElementContent::Leaf(name, definition) => {
                let name_ident = escape_ident(&name.name.to_snake_case());
                let inner = definition.from_xml_impl(ctx);
                quote! {
                    #name_ident: {
                        #inner
                    },
                }
            }
            ElementContent::Leaves(leaves) => {
                let properties = leaves
                    .iter()
                    .map(|el| el.from_xml_impl(ctx))
                    .collect::<Vec<_>>();
                quote!(#(#properties,)*)
            }
        }
    }
}
