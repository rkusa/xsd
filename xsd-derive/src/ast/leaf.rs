use super::{
    ElementDefault, FromXmlImpl, LeafDefinition, Name, Namespaces, State, ToImpl, ToXmlImpl,
};
use proc_macro2::TokenStream;

#[derive(Debug, Clone)]
pub struct Leaf {
    pub name: Name,
    pub definition: LeafDefinition,
}

impl ToImpl for Leaf {
    fn to_impl(&self, state: &mut State) -> TokenStream {
        self.definition.to_impl(state)
    }
}

impl ToXmlImpl for Leaf {
    fn to_xml_impl(&self, element_default: &ElementDefault) -> TokenStream {
        self.definition.to_xml_impl(element_default)
    }
}

impl FromXmlImpl for Leaf {
    fn from_xml_impl<'a>(
        &self,
        element_default: &ElementDefault,
        namespaces: &'a Namespaces<'a>,
    ) -> TokenStream {
        self.definition.from_xml_impl(element_default, namespaces)
    }
}
