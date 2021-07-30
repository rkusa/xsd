use super::{LiteralType, Name};
use crate::ast::Root;
use crate::xsd::context::SchemaContext;
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug, Clone)]
pub enum LeafContent {
    Literal(LiteralType),
    Named(Name),
    Fixed(String),
}

impl LeafContent {
    pub fn to_impl(&self, ctx: &SchemaContext) -> TokenStream {
        match self {
            LeafContent::Literal(literal) => literal.to_impl(),
            LeafContent::Named(name) => match ctx.elements.get(name) {
                Some(&Root::Leaf(ref def)) => def.to_impl(ctx),
                _ => name.to_impl(),
            },
            LeafContent::Fixed(_) => quote!(()),
        }
    }

    pub fn to_xml_impl(&self, ctx: &SchemaContext) -> TokenStream {
        match self {
            LeafContent::Literal(literal) => {
                let inner = literal.to_xml_impl();
                quote! {
                    let val = #inner;
                    if !val.is_empty() {
                        writer.write(XmlEvent::characters(&val))?;
                    }
                }
            }
            LeafContent::Named(name) => match ctx.elements.get(name) {
                Some(&Root::Leaf(ref def)) => def.to_xml_impl(ctx),
                _ => name.to_xml_impl(),
            },
            LeafContent::Fixed(fixed) => quote! {
                writer.write(XmlEvent::characters(#fixed))?;
            },
        }
    }

    pub fn from_xml_impl(&self, ctx: &SchemaContext) -> TokenStream {
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
            LeafContent::Named(name) => match ctx.elements.get(name) {
                Some(&Root::Leaf(ref def)) => def.from_xml_impl(ctx),
                _ => name.from_xml_impl(),
            },
            LeafContent::Fixed(fixed) => {
                quote! {
                    {
                        let val = node.text()?;
                        if val != #fixed {
                            return Err(::xsd::decode::FromXmlError::FixedMismatch {
                                expected: #fixed,
                                received: val.to_string(),
                            }.into());
                        }
                    }
                }
            }
        }
    }

    pub fn from_str_impl(&self) -> TokenStream {
        match self {
            LeafContent::Literal(literal) => literal.from_str_impl(),
            LeafContent::Named(name) => name.from_str_impl(),
            LeafContent::Fixed(_) => quote! { () },
        }
    }
}
