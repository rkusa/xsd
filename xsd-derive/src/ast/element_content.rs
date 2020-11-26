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
                let inner = leaf.to_impl(state);
                // TODO: prevent collisions between the randomly chosen `value` and attributes that
                // are possibly named `value`
                quote!(pub value: #inner,)
            }
            ElementContent::Leaves(leaves) => {
                let properties = leaves
                    .iter()
                    .map(|el| el.to_impl(state))
                    .collect::<Vec<_>>();
                quote!(#(pub #properties,)*)
            }
        }
    }

    pub fn to_xml_impl(&self, element_default: &ElementDefault) -> TokenStream {
        match &self {
            ElementContent::Leaf(leaf) => {
                let inner = leaf.to_xml_impl(element_default);
                quote! {
                    let val = &self.value;
                    #inner;
                }
            }
            ElementContent::Leaves(leaves) => {
                let properties = leaves
                    .iter()
                    .map(|el| el.to_xml_impl(element_default))
                    .collect::<Vec<_>>();
                quote! {
                    writer.write(start)?;
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
                    value: {
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
