use super::{get_xml_name, ElementDefault, LeafContent, Name, Namespaces, State};
use crate::generator::escape_ident;
use inflector::Inflector;
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug, Clone)]
pub struct Attribute {
    pub name: Name,
    pub content: LeafContent,
}

impl Attribute {
    pub fn to_impl(&self, state: &mut State) -> TokenStream {
        let name_ident = escape_ident(&self.name.name.to_snake_case());
        let type_ident = self.content.to_impl(state);
        quote! { pub #name_ident: #type_ident, }
    }

    pub fn to_xml_impl(&self, element_default: &ElementDefault) -> TokenStream {
        let name_ident = escape_ident(&self.name.name.to_snake_case());
        let name_xml = get_xml_name(&self.name, element_default.qualified);
        let inner = match &self.content {
            LeafContent::Literal(literal) => literal.to_xml_impl(element_default),
            LeafContent::Named(_) => quote! { val.as_str() },
        };
        quote! {
            let val = &self.#name_ident;
            let val = #inner;
            let start = start.attr(#name_xml, &val);
        }
    }

    pub fn from_xml_impl<'a>(
        &self,
        _element_default: &ElementDefault,
        _namespaces: &'a Namespaces<'a>,
    ) -> TokenStream {
        let name_ident = escape_ident(&self.name.name.to_snake_case());
        let name_xml = &self.name.name;
        let inner = self.content.from_str_impl();
        quote! {
            #name_ident: {
                let val = node.try_attribute(#name_xml)?;
                #inner
            }
        }
    }
}
