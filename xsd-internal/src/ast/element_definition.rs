use crate::xsd::context::SchemaContext;

use super::{Attribute, ElementContent, LeafContent, LeafDefinition};
use proc_macro2::TokenStream;
use quote::quote;
use quote::TokenStreamExt;

#[derive(Debug, Clone)]
pub struct ElementDefinition {
    pub attributes: Vec<Attribute>,
    pub content: Option<ElementContent>,
    pub is_virtual: bool,
    pub docs: Option<String>,
}

impl ElementDefinition {
    pub fn with_docs(mut self, docs: Option<String>) -> Self {
        if docs.is_some() {
            self.docs = docs
        }
        self
    }

    pub fn to_impl(&self, ctx: &SchemaContext) -> TokenStream {
        let mut ts = TokenStream::new();
        if let Some(content) = &self.content {
            ts.append_all(content.to_impl(ctx));
        }
        for attr in &self.attributes {
            ts.append_all(attr.to_impl());
        }
        ts
    }

    pub fn to_xml_impl(&self, ctx: &SchemaContext) -> TokenStream {
        let mut ts = TokenStream::new();
        for attr in &self.attributes {
            ts.append_all(attr.to_xml_impl(ctx));
        }
        let is_named_leaf = matches!(
            self.content,
            Some(ElementContent::Leaf(LeafDefinition {
                content: LeafContent::Named(_),
                ..
            }))
        );
        let wrap = !self.is_virtual && !is_named_leaf;
        if wrap {
            ts.append_all(quote! { ctx.write_start_element(writer)?; });
        }
        if let Some(content) = &self.content {
            ts.append_all(content.to_xml_impl(ctx));
        }
        if wrap {
            ts.append_all(quote! { ctx.write_end_element(writer)?; });
        }
        ts
    }

    pub fn from_xml_impl(&self, ctx: &SchemaContext) -> TokenStream {
        let mut ts = TokenStream::new();
        if let Some(content) = &self.content {
            ts.append_all(content.from_xml_impl(ctx));
        }
        for attr in &self.attributes {
            ts.append_all(attr.from_xml_impl());
        }
        quote! {
            {
                #ts
            }
        }
    }
}
