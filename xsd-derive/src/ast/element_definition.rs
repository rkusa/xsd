use super::{ElementContent, ElementDefault, FromXmlImpl, Namespaces, State, ToImpl, ToXmlImpl};
use proc_macro2::TokenStream;

#[derive(Debug, Clone)]
pub struct ElementDefinition {
    // attrs
    pub kind: Kind,
    pub content: ElementContent,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Kind {
    Root,
    Child,
    Virtual,
}

impl ToImpl for ElementDefinition {
    fn to_impl(&self, state: &mut State) -> TokenStream {
        self.content.to_impl(state)
    }
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
