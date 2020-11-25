use super::{
    ElementDefault, ElementDefinition, FromXmlImpl, LeafDefinition, Namespaces, ToXmlImpl,
};
use super::{State, ToImpl};
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug, Clone)]
pub enum Root {
    Leaf(LeafDefinition),
    Element(ElementDefinition),
}

impl ToImpl for Root {
    fn to_impl(&self, state: &mut State) -> TokenStream {
        match self {
            Root::Leaf(def) => {
                let inner = def.to_impl(state);
                quote! {
                    (pub #inner);
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
        }
    }
}

impl ToXmlImpl for Root {
    fn to_xml_impl(&self, element_default: &ElementDefault) -> TokenStream {
        match self {
            Root::Leaf(def) => {
                let inner = def.to_xml_impl(element_default);
                quote! {
                    let val = &self.0;
                    #inner
                }
            }
            Root::Element(def) => def.to_xml_impl(element_default),
        }
    }
}

impl FromXmlImpl for Root {
    fn from_xml_impl<'a>(
        &self,
        element_default: &ElementDefault,
        namespaces: &'a Namespaces<'a>,
    ) -> TokenStream {
        match self {
            Root::Leaf(def) => {
                let inner = def.from_xml_impl(element_default, namespaces);
                quote! {
                    (#inner)
                }
            }
            Root::Element(def) => def.from_xml_impl(element_default, namespaces),
        }
    }
}
