use super::{Attribute, ElementContent, ElementDefault, Namespaces, State};
use proc_macro2::TokenStream;
use quote::quote;
use quote::TokenStreamExt;

#[derive(Debug, Clone)]
pub struct ElementDefinition {
    pub attributes: Vec<Attribute>,
    pub content: Option<ElementContent>,
    pub is_virtual: bool,
}

impl ElementDefinition {
    pub fn to_impl(&self, state: &mut State) -> TokenStream {
        let mut ts = TokenStream::new();
        if let Some(content) = &self.content {
            ts.append_all(content.to_impl(state));
        }
        for attr in &self.attributes {
            ts.append_all(attr.to_impl(state));
        }
        ts
    }

    pub fn to_xml_impl(&self, element_default: &ElementDefault) -> TokenStream {
        let mut ts = TokenStream::new();
        for attr in &self.attributes {
            ts.append_all(attr.to_xml_impl(element_default));
        }
        if let Some(content) = &self.content {
            // TODO: rework when element starts are written to make it easier to understand and
            // extend ...
            if !self.is_virtual && matches!(content, ElementContent::Leaves(_)) {
                ts.append_all(quote! { writer.write(start)?; });
            }
            ts.append_all(content.to_xml_impl(element_default));
        } else {
            ts.append_all(quote! {
                writer.write(start)?;
            });
        }
        ts
    }

    pub fn from_xml_impl<'a>(
        &self,
        element_default: &ElementDefault,
        namespaces: &'a Namespaces<'a>,
    ) -> TokenStream {
        let mut ts = TokenStream::new();
        if let Some(content) = &self.content {
            ts.append_all(content.from_xml_impl(element_default, namespaces));
        }
        for attr in &self.attributes {
            ts.append_all(attr.from_xml_impl(element_default, namespaces));
        }
        quote! {
            {
                #ts
            }
        }
    }
}
