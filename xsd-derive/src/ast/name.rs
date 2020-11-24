use crate::generator::escape_ident;

use super::{ElementDefault, FromXmlImpl, Namespaces, State, ToImpl, ToXmlImpl};
use inflector::Inflector;
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Namespace {
    None,
    Target,
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Name {
    pub name: String,
    pub namespace: Namespace,
}

impl Name {
    pub fn new(name: impl Into<String>, namespace: Namespace) -> Self {
        Name {
            name: name.into(),
            namespace,
        }
    }
}

impl ToImpl for Name {
    fn to_impl(&self, _state: &mut State) -> TokenStream {
        let name_ident = escape_ident(&self.name.to_pascal_case());
        quote!(#name_ident)
    }
}

impl ToXmlImpl for Name {
    fn to_xml_impl(&self, _element_default: &ElementDefault) -> TokenStream {
        quote! {
            val.to_xml_writer(writer)?;
        }
    }
}

impl FromXmlImpl for Name {
    fn from_xml_impl<'a>(
        &self,
        _element_default: &ElementDefault,
        _namespaces: &'a Namespaces<'a>,
    ) -> TokenStream {
        let name_ident = escape_ident(&self.name.to_pascal_case());
        quote! {
            #name_ident::from_xml_node(&node)?
        }
    }
}

impl FromXmlImpl for Namespace {
    fn from_xml_impl<'a>(
        &self,
        element_default: &ElementDefault,
        _namespaces: &'a Namespaces<'a>,
    ) -> TokenStream {
        let namespace = match self {
            Namespace::None => None,
            Namespace::Target => element_default.target_namespace.as_deref(),
            Namespace::Other(ns) => Some(ns.as_str()),
        };
        match namespace {
            Some(ns) => quote!(Some(#ns)),
            None => quote!(None),
        }
    }
}
