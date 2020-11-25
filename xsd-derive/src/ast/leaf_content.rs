use super::{ElementDefault, FromXmlImpl, LiteralType, Name, Namespaces, ToXmlImpl};
use super::{State, ToImpl};
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug, Clone)]
pub enum LeafContent {
    Literal(LiteralType),
    Named(Name),
}

impl ToImpl for LeafContent {
    fn to_impl(&self, state: &mut State) -> TokenStream {
        match self {
            LeafContent::Literal(literal) => literal.to_impl(state),
            LeafContent::Named(name) => name.to_impl(state),
        }
    }
}

impl ToXmlImpl for LeafContent {
    fn to_xml_impl(&self, element_default: &ElementDefault) -> TokenStream {
        match self {
            LeafContent::Literal(literal) => {
                let inner = literal.to_xml_impl(element_default);
                quote! {
                    writer.write(start)?;
                    let val = #inner;
                    writer.write(XmlEvent::characters(&val))?;
                }
            }
            LeafContent::Named(name) => name.to_xml_impl(element_default),
        }
    }
}

impl FromXmlImpl for LeafContent {
    fn from_xml_impl<'a>(
        &self,
        element_default: &ElementDefault,
        namespaces: &'a Namespaces<'a>,
    ) -> TokenStream {
        match self {
            LeafContent::Literal(literal) => literal.from_xml_impl(element_default, namespaces),
            LeafContent::Named(name) => name.from_xml_impl(element_default, namespaces),
        }
    }
}
