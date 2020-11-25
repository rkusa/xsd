use super::{
    Attribute, ElementContent, ElementDefault, FromXmlImpl, Namespaces, State, ToImpl, ToXmlImpl,
};
use proc_macro2::TokenStream;
use quote::quote;
use quote::TokenStreamExt;

#[derive(Debug, Clone)]
pub struct ElementDefinition {
    pub kind: Kind,
    pub attributes: Vec<Attribute>,
    pub content: ElementContent,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Kind {
    Root,
    Child,
    Virtual,
}

impl ToImpl for ElementDefinition {
    fn to_impl(&self, state: &mut State) -> TokenStream {
        let mut ts = TokenStream::new();
        ts.append_all(self.content.to_impl(state));
        for attr in &self.attributes {
            ts.append_all(attr.to_impl(state));
        }
        quote! {
            {
                #ts
            }
        }
    }
}

impl ToXmlImpl for ElementDefinition {
    fn to_xml_impl(&self, element_default: &ElementDefault) -> TokenStream {
        let mut ts = TokenStream::new();
        for attr in &self.attributes {
            ts.append_all(attr.to_xml_impl(element_default));
        }
        ts.append_all(quote! { writer.write(start)?; });
        ts.append_all(self.content.to_xml_impl(element_default));
        ts
    }
}

impl FromXmlImpl for ElementDefinition {
    fn from_xml_impl<'a>(
        &self,
        element_default: &ElementDefault,
        namespaces: &'a Namespaces<'a>,
    ) -> TokenStream {
        let mut ts = TokenStream::new();
        ts.append_all(self.content.from_xml_impl(element_default, namespaces));
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
