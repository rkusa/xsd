use super::{get_xml_name, ElementDefault, FromXmlImpl, LiteralType, Name, Namespaces, ToXmlImpl};
use super::{State, ToImpl};
use crate::generator::escape_ident;
use inflector::Inflector;
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug, Clone)]
pub struct Attribute {
    pub name: Name,
    pub content: LiteralType,
}

impl ToImpl for Attribute {
    fn to_impl(&self, state: &mut State) -> TokenStream {
        let name_ident = escape_ident(&self.name.name.to_snake_case());
        let type_ident = self.content.to_impl(state);
        quote! { pub #name_ident: #type_ident, }
    }
}

impl ToXmlImpl for Attribute {
    fn to_xml_impl(&self, element_default: &ElementDefault) -> TokenStream {
        let name_ident = escape_ident(&self.name.name.to_snake_case());
        let name_xml = get_xml_name(&self.name, element_default.qualified);
        let inner = self.content.to_xml_impl(element_default);
        quote! {
            let val = {
                let val = &self.#name_ident;
                #inner
            };
            let start = start.attr(#name_xml, &val);
        }
    }
}

impl FromXmlImpl for Attribute {
    fn from_xml_impl<'a>(
        &self,
        element_default: &ElementDefault,
        namespaces: &'a Namespaces<'a>,
    ) -> TokenStream {
        let name_ident = escape_ident(&self.name.name.to_snake_case());
        let name_xml = &self.name.name;
        let inner = self.content.from_xml_impl(element_default, namespaces);
        quote! {
            #name_ident: {
                let val = node.try_attribute(#name_xml)?;
                #inner
            }
        }
    }
}
