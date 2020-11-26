use super::{ElementDefault, LeafContent, Namespaces, State};
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug, Clone)]
pub struct LeafDefinition {
    pub content: LeafContent,
    // pub docs: Option<String>,
    pub restrictions: Vec<Restriction>,
    // pub fixed: Option<String>,
    pub min_occurs: MinOccurs,
    pub max_occurs: MaxOccurs,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct MinOccurs(pub u32);

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MaxOccurs {
    Number(u32),
    Unbounded,
}

pub type Restriction = ();
// #[derive(Debug, PartialEq, Clone)]
// pub enum Restriction {
//     // MinLength(usize),
//     // MaxLength(usize),
//     // Pattern(String),
//     // MinInclusive(Decimal),
//     // MaxInclusive(Decimal),
//     // FractionDigits(usize),
//     // TotalDigits(usize),
// }

impl LeafDefinition {
    pub fn is_optional(&self) -> bool {
        self.min_occurs == MinOccurs(0)
    }

    fn is_vec(&self) -> bool {
        match self.max_occurs {
            MaxOccurs::Unbounded => true,
            MaxOccurs::Number(n) if n > 1 => true,
            _ => false,
        }
    }

    pub fn to_impl(&self, state: &mut State) -> TokenStream {
        let inner = self.content.to_impl(state);
        if self.is_optional() {
            quote! { Option<#inner> }
        } else {
            inner
        }
    }

    pub fn to_xml_impl(&self, element_default: &ElementDefault) -> TokenStream {
        let inner = self.content.to_xml_impl(element_default);
        if self.is_optional() {
            quote! {
                if let Some(val) = val {
                    #inner
                }
            }
        } else {
            inner
        }
    }

    pub fn from_xml_impl<'a>(
        &self,
        element_default: &ElementDefault,
        namespaces: &'a Namespaces<'a>,
    ) -> TokenStream {
        let inner = self.content.from_xml_impl(element_default, namespaces);
        if self.is_optional() {
            quote! {
                if let Some(node) = node.take() {
                    let val = node.text()?;
                    Some(#inner)
                } else {
                    None
                }
            }
        } else {
            quote! {
                {
                    let node = node.try_take()?;
                    let val = node.text()?;
                    #inner
                }
            }
        }
    }
}

impl Default for MinOccurs {
    fn default() -> Self {
        MinOccurs(1)
    }
}

impl Default for MaxOccurs {
    fn default() -> Self {
        MaxOccurs::Number(1)
    }
}
