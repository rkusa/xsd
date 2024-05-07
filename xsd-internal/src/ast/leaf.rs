use super::{LeafContent, LeafDefinition, Name};
use crate::ast::Root;
use crate::utils::escape_ident;
use crate::xsd::context::SchemaContext;
use inflector::Inflector;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, TokenStreamExt};

#[derive(Debug, Clone)]
pub struct Leaf {
    pub name: Name,
    pub definition: LeafDefinition,
    pub is_unordered: bool,
    pub is_virtual: bool,
    pub min_occurs: MinOccurs,
    pub max_occurs: MaxOccurs,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct MinOccurs(pub u32);

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MaxOccurs {
    Number(u32),
    Unbounded,
}

impl MaxOccurs {
    pub fn is_vec(&self) -> bool {
        match self {
            MaxOccurs::Number(n) => *n > 1,
            MaxOccurs::Unbounded => true,
        }
    }
}

impl Leaf {
    pub fn is_optional(&self) -> bool {
        self.min_occurs == MinOccurs(0)
    }

    pub fn is_vec(&self) -> bool {
        match self.max_occurs {
            MaxOccurs::Unbounded => true,
            MaxOccurs::Number(n) if n > 1 => true,
            _ => false,
        }
    }

    pub fn to_impl(&self, ctx: &SchemaContext) -> TokenStream {
        let name_ident = escape_ident(&self.name.name.to_snake_case());
        let mut type_ident = self.definition.to_impl(ctx);
        if self.is_vec() {
            type_ident = quote! { Vec<#type_ident> }
        }
        if self.is_optional() {
            type_ident = quote! { Option<#type_ident> }
        }
        let docs = self
            .definition
            .docs
            .as_deref()
            .filter(|docs| !docs.is_empty())
            .map(|docs| quote! { #[doc = #docs] })
            .unwrap_or_default();
        quote! {
            #docs
            pub #name_ident: #type_ident
        }
    }

    pub fn to_xml_impl(&self, ctx: &SchemaContext) -> TokenStream {
        let name_ident = escape_ident(&self.name.name.to_snake_case());
        let name_xml = ctx.get_xml_element_name(&self.name);
        let inner = self.definition.to_xml_impl(ctx);

        let mut tn = TokenStream::new();
        if self.is_virtual {
            tn.append_all(quote! {
                let mut ctx = ::xsd::Context::wrap(&mut ctx);
            });
        } else {
            tn.append_all(quote! {
                let mut ctx = ::xsd::Context::new(#name_xml);
            });
        }

        let wrap = !self.is_virtual
            && match &self.definition {
                LeafDefinition {
                    content: LeafContent::Named(name),
                    ..
                } => {
                    matches!(
                        ctx.resolve(name),
                        Some(Root::Leaf(LeafDefinition {
                            content: LeafContent::Literal(_),
                            ..
                        }))
                    )
                }
                _ => true,
            };

        if wrap {
            tn.append_all(quote! {
                ctx.write_start_element(writer)?;
            });
        }

        tn.append_all(inner);

        if wrap {
            tn.append_all(quote! {
                ctx.write_end_element(writer)?;
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
            {
                #tn
            }
        }
    }

    pub fn from_xml_impl(&self, ctx: &SchemaContext) -> TokenStream {
        let name_ident = escape_ident(&self.name.name.to_snake_case());
        let name_xml = &self.name.name;
        let namespace_xml = ctx.quote_xml_namespace(&self.name);
        let mut value = self.definition.from_xml_impl(ctx);

        if self.is_virtual {
            if self.is_vec() {
                let name = if let LeafContent::Named(name) = &self.definition.content {
                    name
                } else {
                    // unreachable  ...
                    // TODO: reflect that in the type?
                    unreachable!()
                };

                let first_name = format_ident!("{}", name.name.to_pascal_case());
                value = quote! {
                    {
                        let mut vec = Vec::new();
                        while #first_name::lookahead(node) {
                            vec.push(#value);
                        }
                        vec
                    }
                };

                if self.is_optional() {
                    value = quote! {
                        let val = #value;
                        if val.is_empty() {
                            None
                        } else {
                            Some(val)
                        }
                    }
                }
            } else if self.is_optional() {
                let name = if let LeafContent::Named(name) = &self.definition.content {
                    name
                } else {
                    // unreachable  ...
                    // TODO: reflect that in the type?
                    unreachable!()
                };

                let first_name = format_ident!("{}", name.name.to_pascal_case());
                value = quote! {
                    if #first_name::lookahead(node) {
                        Some(#value)
                    } else {
                        None
                    }
                };
            }
        } else {
            value = if self.is_vec() {
                let mut from_vec = quote! {
                    {
                        let mut vec = Vec::new();
                        while let Some(node) = node.next_child(#name_xml, #namespace_xml) {
                            vec.push(#value);
                        }
                        vec
                    }
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
                    if let Some(node) = node.next_child(#name_xml, #namespace_xml) {
                        Some(#value)
                    } else {
                        None
                    }
                }
            } else if self.is_unordered {
                quote! {
                    let node = node.try_child(#name_xml, #namespace_xml)?;
                    #value
                }
            } else {
                quote! {
                    let node = node.try_next_child(#name_xml, #namespace_xml)?;
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
