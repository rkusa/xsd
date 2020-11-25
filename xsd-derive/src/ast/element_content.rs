use crate::generator::escape_ident;

use super::{
    get_xml_name, ElementDefault, FromXmlImpl, Leaf, LiteralType, Name, Namespaces, ToXmlImpl,
};
use super::{State, ToImpl};
use inflector::Inflector;
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug, Clone)]
pub enum ElementContent {
    Literal(LiteralType),
    Reference(Name),
    Leaves(Vec<Leaf>),
}

impl ToImpl for ElementContent {
    fn to_impl(&self, state: &mut State) -> TokenStream {
        match self {
            ElementContent::Literal(literal) => {
                let inner = literal.to_impl(state);
                quote!(pub value: #inner,)
            }
            ElementContent::Reference(_) => unimplemented!("ElementContent::Reference::to_impl"),
            ElementContent::Leaves(leaves) => {
                let properties = leaves
                    .iter()
                    .map(|el| {
                        let name_ident = escape_ident(&el.name.name.to_snake_case());
                        let type_ident = el.content.to_impl(state);
                        quote! { #name_ident: #type_ident }
                    })
                    .collect::<Vec<_>>();
                quote!(#(pub #properties,)*)
            }
        }
    }
}

impl ToXmlImpl for ElementContent {
    fn to_xml_impl(&self, element_default: &ElementDefault) -> TokenStream {
        match &self {
            ElementContent::Literal(literal) => {
                let inner = literal.to_xml_impl(element_default);
                quote! {
                    let val = &self.value;
                    let val = #inner;
                    writer.write(XmlEvent::characters(&val))?;
                }
            }
            ElementContent::Reference(_) => {
                unimplemented!("ElementContent::Reference::to_xml_impl")
            }
            ElementContent::Leaves(leaves) => {
                let properties = leaves
                    .iter()
                    .map(|el| {
                        let name_ident = escape_ident(&el.name.name.to_snake_case());
                        let name_xml = get_xml_name(&el.name, element_default.qualified);
                        let inner = el.content.to_xml_impl(element_default);
                        quote! {
                            let start = XmlEvent::start_element(#name_xml);
                            let val = &self.#name_ident;
                            #inner
                            writer.write(XmlEvent::end_element())?;
                        }
                    })
                    .collect::<Vec<_>>();
                quote!(#(#properties)*)
            }
        }
    }
}

impl FromXmlImpl for ElementContent {
    fn from_xml_impl<'a>(
        &self,
        element_default: &ElementDefault,
        namespaces: &'a Namespaces<'a>,
    ) -> TokenStream {
        match &self {
            ElementContent::Literal(literal) => {
                let inner = literal.from_xml_impl(element_default, namespaces);
                quote! {
                    value: {
                        let val = node.text()?;
                        #inner
                    },
                }
            }
            ElementContent::Reference(_) => {
                unimplemented!("ElementContent::Reference::from_impl_impl")
            }
            ElementContent::Leaves(leaves) => {
                let properties = leaves
                    .iter()
                    .map(|el| {
                        let name_ident = escape_ident(&el.name.name.to_snake_case());
                        let name_xml = &el.name.name;
                        let namespace_xml = el
                            .name
                            .namespace
                            .from_xml_impl(&element_default, &namespaces);
                        let inner = el.content.from_xml_impl(element_default, namespaces);
                        quote! { #name_ident: {
                            let node = node.try_child(#name_xml, #namespace_xml)?;
                            let val = node.text()?;
                            #inner
                        } }
                    })
                    .collect::<Vec<_>>();
                quote!(#(#properties,)*)
            }
        }
    }
}
