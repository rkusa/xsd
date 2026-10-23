use crate::xsd::context::SchemaContext;

use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Namespace {
    #[default]
    None,
    Id(usize),
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

    pub fn to_impl(&self, ctx: &SchemaContext) -> TokenStream {
        let name_ident = ctx.type_ident(self);
        quote!(#name_ident)
    }

    pub fn to_xml_impl(&self) -> TokenStream {
        quote! {
            val.to_xml_writer(ctx, writer)?;
        }
    }

    pub fn from_xml_impl(&self, ctx: &SchemaContext) -> TokenStream {
        let name_ident = ctx.type_ident(self);
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
