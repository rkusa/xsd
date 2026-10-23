use crate::xsd::context::SchemaContext;

use super::LeafContent;
use proc_macro2::TokenStream;
use rust_decimal::Decimal;

#[derive(Debug, Clone)]
pub struct LeafDefinition {
    pub content: LeafContent,
    // pub docs: Option<String>,
    pub restrictions: Vec<Restriction>,
    /// The value to use if the element is present but empty.
    pub default: Option<String>,
    pub docs: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Restriction {
    Length(usize),
    MinLength(usize),
    MaxLength(usize),
    Pattern(String),
    MinInclusive(Decimal),
    MaxInclusive(Decimal),
    FractionDigits(usize),
    TotalDigits(usize),
}

impl LeafDefinition {
    pub fn to_impl(&self, ctx: &SchemaContext) -> TokenStream {
        self.content.to_impl(ctx)
    }

    pub fn to_xml_impl(&self, ctx: &SchemaContext) -> TokenStream {
        self.content.to_xml_impl(ctx)
    }

    pub fn from_xml_impl(&self, ctx: &SchemaContext) -> TokenStream {
        self.content.from_xml_impl(ctx, self.default.as_deref())
    }
}
