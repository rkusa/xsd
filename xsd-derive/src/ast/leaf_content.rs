use super::{ElementDefault, LiteralType, Name, Namespaces, State};
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug, Clone)]
pub enum LeafContent {
    Literal(LiteralType),
    Named(Name),
}

impl LeafContent {
    pub fn to_impl(&self, state: &mut State) -> TokenStream {
        match self {
            LeafContent::Literal(literal) => literal.to_impl(state),
            LeafContent::Named(name) => name.to_impl(state),
        }
    }

    pub fn to_xml_impl(&self, element_default: &ElementDefault) -> TokenStream {
        match self {
            LeafContent::Literal(literal) => {
                let inner = literal.to_xml_impl(element_default);
                quote! {
                    let val = #inner;
                    if !val.is_empty() {
                        writer.write(XmlEvent::characters(&val))?;
                    }
                }
            }
            LeafContent::Named(name) => name.to_xml_impl(element_default),
        }
    }

    pub fn from_xml_impl<'a>(
        &self,
        element_default: &ElementDefault,
        namespaces: &'a Namespaces<'a>,
    ) -> TokenStream {
        match self {
            LeafContent::Literal(literal) => {
                let inner = literal.from_str_impl();
                quote! {
                    {
                        let val = node.text()?;
                        #inner
                    }
                }
            }
            LeafContent::Named(name) => name.from_xml_impl(element_default, namespaces),
        }
    }

    pub fn from_str_impl(&self) -> TokenStream {
        match self {
            LeafContent::Literal(literal) => literal.from_str_impl(),
            LeafContent::Named(name) => name.from_str_impl(),
        }
    }
}
