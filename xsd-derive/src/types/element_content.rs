use super::{Element, ElementDefault, FromXmlImpl, Literal, Namespaces, ToXmlImpl};
use super::{State, ToImpl};
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug, Clone)]
pub enum ElementContent {
    Literal(Literal),
    Elements(Vec<Element>),
}

impl ToImpl for ElementContent {
    fn to_impl(&self, state: &mut State) -> TokenStream {
        match &self {
            ElementContent::Literal(literal) => {
                let inner = literal.to_impl(state);
                quote! {
                    (pub #inner);
                }
            }
            ElementContent::Elements(_) => {
                quote! { {} }
            }
        }
    }
}

impl ToXmlImpl for ElementContent {
    fn to_xml_impl(&self, element_default: &ElementDefault) -> TokenStream {
        match &self {
            ElementContent::Literal(literal) => {
                let inner = literal.to_xml_impl(element_default);
                quote! {
                    let val = &self.0;
                    #inner
                }
            }
            ElementContent::Elements(_) => {
                unimplemented!("ToXmlImpl for ElementContent::Elements")
            }
        }
    }
}

impl FromXmlImpl for ElementContent {
    fn from_xml_impl<'a>(
        &self,
        element_default: &ElementDefault,
        namespaces: &'a Namespaces<'a>,
    ) -> TokenStream {
        match &self {
            ElementContent::Literal(literal) => {
                let inner = literal.from_xml_impl(element_default, namespaces);
                quote! { (#inner) }
            }
            ElementContent::Elements(_) => {
                unimplemented!("FromXmlImpl for ElementContent::Elements")
            }
        }
    }
}
