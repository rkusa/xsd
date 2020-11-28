use super::{ElementDefault, Leaf, LeafDefinition, Namespaces, State};
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug, Clone)]
pub enum ElementContent {
    Leaf(LeafDefinition),
    Leaves(Vec<Leaf>),
}

impl ElementContent {
    pub fn to_impl(&self, state: &mut State) -> TokenStream {
        match self {
            ElementContent::Leaf(leaf) => {
                let docs = leaf
                    .docs
                    .as_ref()
                    .map(|docs| quote! { #[doc = #docs] })
                    .unwrap_or_else(TokenStream::new);
                let inner = leaf.to_impl(state);
                // TODO: prevent collisions between the randomly chosen `pub value_` and attributes that
                // are possibly named `pub value_`
                quote! {
                    #docs
                    pub value_: #inner,
                }
            }
            ElementContent::Leaves(leaves) => {
                let properties = leaves
                    .iter()
                    .map(|el| el.to_impl(state))
                    .collect::<Vec<_>>();
                quote!(#(#properties,)*)
            }
        }
    }

    pub fn to_xml_impl(&self, element_default: &ElementDefault) -> TokenStream {
        match &self {
            ElementContent::Leaf(leaf) => {
                let inner = leaf.to_xml_impl(element_default);
                quote! {
                    let val = &self.value_;
                    #inner;
                }
            }
            ElementContent::Leaves(leaves) => {
                let properties = leaves
                    .iter()
                    .map(|el| el.to_xml_impl(element_default))
                    .collect::<Vec<_>>();
                quote! {
                    #(#properties)*
                }
            }
        }
    }

    pub fn from_xml_impl<'a>(
        &self,
        element_default: &ElementDefault,
        namespaces: &'a Namespaces<'a>,
    ) -> TokenStream {
        match &self {
            ElementContent::Leaf(leaf) => {
                let inner = leaf.from_xml_impl(element_default, namespaces);
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
                    .map(|el| el.from_xml_impl(element_default, namespaces))
                    .collect::<Vec<_>>();
                quote!(#(#properties,)*)
            }
        }
    }
}
