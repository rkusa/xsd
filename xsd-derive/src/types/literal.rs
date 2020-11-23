use super::{ElementDefault, FromXmlImpl, LiteralType, Namespaces, ToXmlImpl};
use super::{State, ToImpl};
use proc_macro2::TokenStream;

#[derive(Debug, Clone)]
pub struct Literal {
    pub type_: LiteralType,
    // pub docs: Option<String>,
    // pub restrictions: Option<Vec<Restriction>>,
    // pub fixed: Option<String>,
}

impl ToImpl for Literal {
    fn to_impl(&self, state: &mut State) -> TokenStream {
        self.type_.to_impl(state)
    }
}

impl ToXmlImpl for Literal {
    fn to_xml_impl(&self, element_default: &ElementDefault) -> TokenStream {
        self.type_.to_xml_impl(element_default)
    }
}

impl FromXmlImpl for Literal {
    fn from_xml_impl<'a>(
        &self,
        element_default: &ElementDefault,
        namespaces: &'a Namespaces<'a>,
    ) -> TokenStream {
        self.type_.from_xml_impl(element_default, namespaces)
    }
}
