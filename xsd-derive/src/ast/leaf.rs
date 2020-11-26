use super::{get_xml_name, ElementDefault, LeafDefinition, Name, Namespaces, State};
use crate::generator::escape_ident;
use inflector::Inflector;
use proc_macro2::TokenStream;
use quote::{quote, TokenStreamExt};

#[derive(Debug, Clone)]
pub struct Leaf {
    pub name: Name,
    pub definition: LeafDefinition,
    pub is_virtual: bool,
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

impl Leaf {
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
        let name_ident = escape_ident(&self.name.name.to_snake_case());
        let mut type_ident = self.definition.to_impl(state);
        if self.is_vec() {
            type_ident = quote! { Vec<#type_ident> }
        }
        if self.is_optional() {
            type_ident = quote! { Option<#type_ident> }
        }
        quote! { #name_ident: #type_ident }
    }

    pub fn to_xml_impl(&self, element_default: &ElementDefault) -> TokenStream {
        let name_ident = escape_ident(&self.name.name.to_snake_case());
        let name_xml = get_xml_name(&self.name, element_default.qualified);
        let inner = self.definition.to_xml_impl(element_default);

        let mut tn = TokenStream::new();
        // We don't really want to create a wrapping element here for virtual leaves but still
        // require a `start` var even though that it will not be used.
        // TODO: is there a better way?
        tn.append_all(quote! {
            let start = XmlEvent::start_element(#name_xml);
            #inner
        });

        if !self.is_virtual {
            tn.append_all(quote! {
                writer.write(XmlEvent::end_element())?;
            })
        }

        if self.is_vec() {
            tn = quote! {
                for val in val {
                    #tn
                }
            };
        }
        if self.is_optional() {
            tn = quote! {
                if let Some(val) = val {
                    #tn
                }
            };
        }

        quote! {
            let val = &self.#name_ident;
            #tn
        }
    }

    pub fn from_xml_impl<'a>(
        &self,
        element_default: &ElementDefault,
        namespaces: &'a Namespaces<'a>,
    ) -> TokenStream {
        let name_ident = escape_ident(&self.name.name.to_snake_case());
        let name_xml = &self.name.name;
        let namespace_xml = self
            .name
            .namespace
            .from_xml_impl(&element_default, &namespaces);
        let mut value = self.definition.from_xml_impl(element_default, namespaces);

        if !self.is_virtual {
            value = if self.is_vec() {
                let mut from_vec = quote! {
                    node
                        .children(#name_xml, #namespace_xml)
                        .map(|node| {
                            let val = node.text()?;
                            Ok(#value)
                        })
                        .collect::<Result<Vec<_>, ::xsd::decode::FromXmlError>>()?
                };

                if self.is_optional() {
                    from_vec = quote! {
                        let val = #from_vec;
                        if val.is_empty() {
                            None
                        } else {
                            Some(val)
                        }
                    }
                }

                from_vec
            } else if self.is_optional() {
                quote! {
                    if let Some(node) = node.child(#name_xml, #namespace_xml) {
                        Some(#value)
                    } else {
                        None
                    }
                }
            } else {
                quote! {
                    let node = node.try_child(#name_xml, #namespace_xml)?;
                    #value
                }
            };
        }

        quote! {
           #name_ident: { #value }
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
