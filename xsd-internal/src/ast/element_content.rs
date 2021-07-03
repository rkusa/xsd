use crate::xsd::context::SchemaContext;

use super::{Leaf, LeafDefinition};
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug, Clone)]
pub enum ElementContent {
    Leaf(LeafDefinition),
    Leaves(Vec<Leaf>),
}

impl ElementContent {
    pub fn to_impl(&self, ctx: &SchemaContext) -> TokenStream {
        match self {
            ElementContent::Leaf(leaf) => {
                let docs = leaf
                    .docs
                    .as_ref()
                    .map(|docs| quote! { #[doc = #docs] })
                    .unwrap_or_else(TokenStream::new);
                let inner = leaf.to_impl();
                // TODO: prevent collisions between the randomly chosen `pub value_` and attributes that
                // are possibly named `pub value_`
                quote! {
                    #docs
                    pub value_: #inner,
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
            ElementContent::Leaf(leaf) => {
                let inner = leaf.to_xml_impl();
                quote! {
                    let val = &self.value_;
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
            ElementContent::Leaf(leaf) => {
                let inner = leaf.from_xml_impl();
                quote! {
                    value_: {
                        let val = node.text()?;
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
