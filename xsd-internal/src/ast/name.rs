use crate::utils::escape_ident;

use super::{ElementDefault, Namespaces, State};
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

impl Name {
    pub fn to_impl(&self, _state: &mut State) -> TokenStream {
        let name_ident = escape_ident(&self.name.to_pascal_case());
        quote!(#name_ident)
    }

    pub fn to_xml_impl(&self, _element_default: &ElementDefault) -> TokenStream {
        quote! {
            val.to_xml_writer(ctx, writer)?;
        }
    }

    pub fn from_xml_impl<'a>(
        &self,
        _element_default: &ElementDefault,
        _namespaces: &'a Namespaces<'a>,
    ) -> TokenStream {
        let name_ident = escape_ident(&self.name.to_pascal_case());
        quote! {
            #name_ident::from_xml_node(&node)?
        }
    }

    pub fn from_str_impl(&self) -> TokenStream {
        quote! {
            ::std::str::FromStr::from_str(val)?
        }
    }
}

impl Namespace {
    pub fn to_quote(&self, element_default: &ElementDefault) -> TokenStream {
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
