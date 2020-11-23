use super::{ElementContent, ElementDefault, FromXmlImpl, LiteralType, Namespaces, ToXmlImpl};
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug, Clone)]
pub struct ElementDefinition {
    // attrs
    pub content: ElementContent,
}

impl ToXmlImpl for ElementDefinition {
    fn to_xml_impl(&self, element_default: &ElementDefault) -> TokenStream {
        self.content.to_xml_impl(element_default)
    }
}

impl FromXmlImpl for ElementDefinition {
    fn from_xml_impl<'a>(
        &self,
        element_default: &ElementDefault,
        namespaces: &'a Namespaces<'a>,
    ) -> TokenStream {
        self.content.from_xml_impl(element_default, namespaces)
    }
}
