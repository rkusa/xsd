use super::{ElementDefault, FromXmlImpl, LeafContent, Namespaces, State, ToImpl, ToXmlImpl};
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug, Clone)]
pub struct LeafDefinition {
    pub content: LeafContent,
    // pub docs: Option<String>,
    pub restrictions: Vec<Restriction>,
    // pub fixed: Option<String>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Restriction {
    // MinLength(usize),
    // MaxLength(usize),
    // Pattern(String),
    // MinInclusive(Decimal),
    // MaxInclusive(Decimal),
    // FractionDigits(usize),
    // TotalDigits(usize),
    Enum(Vec<String>),
}

impl ToImpl for LeafDefinition {
    fn to_impl(&self, state: &mut State) -> TokenStream {
        self.content.to_impl(state)
    }
}

impl ToXmlImpl for LeafDefinition {
    fn to_xml_impl(&self, element_default: &ElementDefault) -> TokenStream {
        self.content.to_xml_impl(element_default)
    }
}

impl FromXmlImpl for LeafDefinition {
    fn from_xml_impl<'a>(
        &self,
        element_default: &ElementDefault,
        namespaces: &'a Namespaces<'a>,
    ) -> TokenStream {
        let inner = self.content.from_xml_impl(element_default, namespaces);
        quote! {
            {
                let val = node.text()?;
                #inner
            }
        }
    }
}
