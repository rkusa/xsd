use crate::generator::escape_ident;

use super::{get_xml_name, ElementDefault, Leaf, LeafDefinition, Namespaces, State};
use inflector::Inflector;
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
                    .map(|el| {
                        let name_ident = escape_ident(&el.name.name.to_snake_case());
                        let type_ident = el.definition.to_impl(state);
                        quote! { #name_ident: #type_ident }
                    })
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
                    .map(|el| {
                        let name_ident = escape_ident(&el.name.name.to_snake_case());
                        let name_xml = get_xml_name(&el.name, element_default.qualified);
                        let inner = el.definition.to_xml_impl(element_default);
                        if el.is_virtual {
                            // We don't really want to create a wrapping element here but still
                            // require a `start` var even though that it will not be used.
                            // TODO: is there a better way?
                            quote! {
                                let start = XmlEvent::start_element(#name_xml);
                                let val = &self.#name_ident;
                                #inner
                            }
                        } else {
                            quote! {
                                let start = XmlEvent::start_element(#name_xml);
                                let val = &self.#name_ident;
                                #inner
                                writer.write(XmlEvent::end_element())?;
                            }
                        }
                    })
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
                    .map(|el| {
                        let name_ident = escape_ident(&el.name.name.to_snake_case());
                        let name_xml = &el.name.name;
                        let namespace_xml = el
                            .name
                            .namespace
                            .from_xml_impl(&element_default, &namespaces);
                        let inner = el.definition.from_xml_impl(element_default, namespaces);

                        if el.is_virtual {
                            quote! {
                               #name_ident: #inner
                            }
                        } else {
                            quote! {
                               #name_ident: {
                                   let node = node.try_child(#name_xml, #namespace_xml)?;
                                   let val = node.text()?;
                                   #inner
                               }
                            }
                        }
                    })
                    .collect::<Vec<_>>();
                quote!(#(#properties,)*)
            }
        }
    }
}
