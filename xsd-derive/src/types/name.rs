use super::{ElementDefault, FromXmlImpl, Namespaces};
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
