use super::{ElementDefault, LeafContent, Namespaces, State};
use proc_macro2::TokenStream;
use rust_decimal::Decimal;

#[derive(Debug, Clone)]
pub struct LeafDefinition {
    pub content: LeafContent,
    // pub docs: Option<String>,
    pub restrictions: Vec<Restriction>,
    // pub fixed: Option<String>,
    pub docs: Option<String>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Restriction {
    MinLength(usize),
    MaxLength(usize),
    Pattern(String),
    MinInclusive(Decimal),
    MaxInclusive(Decimal),
    FractionDigits(usize),
    TotalDigits(usize),
}

impl LeafDefinition {
    pub fn to_impl(&self, state: &mut State) -> TokenStream {
        self.content.to_impl(state)
    }

    pub fn to_xml_impl(&self, element_default: &ElementDefault) -> TokenStream {
        self.content.to_xml_impl(element_default)
    }

    pub fn from_xml_impl<'a>(
        &self,
        element_default: &ElementDefault,
        namespaces: &'a Namespaces<'a>,
    ) -> TokenStream {
        self.content.from_xml_impl(element_default, namespaces)
    }
}
