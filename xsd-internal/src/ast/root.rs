use std::collections::HashMap;

use crate::xsd::context::SchemaContext;

use super::{ElementContent, ElementDefinition, Leaf, LeafContent, LeafDefinition, Name};
use inflector::Inflector;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, TokenStreamExt};
use syn::Ident;

#[derive(Debug, Clone)]
pub enum Root {
    Leaf(LeafDefinition),
    Enum(Vec<Name>),
    Element(ElementDefinition),
    Choice(ChoiceDefinition),
}

#[derive(Debug, Clone)]
pub struct ChoiceDefinition {
    pub variants: Vec<Leaf>,
    pub is_virtual: bool,
    pub docs: Option<String>,
}

impl Root {
    pub fn with_docs(mut self, docs: Option<String>) -> Self {
        if docs.is_some() {
            match &mut self {
                Root::Leaf(def) => def.docs = docs,
                Root::Enum(_) => {}
                Root::Element(def) => def.docs = docs,
                Root::Choice(def) => def.docs = docs,
            }
        }
        self
    }

    pub fn is_enum(&self) -> bool {
        matches!(self, Root::Enum(_) | Root::Choice(_))
    }

    pub fn docs(&self) -> Option<&str> {
        match self {
            Root::Leaf(def) => def.docs.as_deref(),
            Root::Enum(_) => None,
            Root::Element(def) => def.docs.as_deref(),
            Root::Choice(def) => def.docs.as_deref(),
        }
    }

    pub fn to_declaration(&self, root_name: &Ident, ctx: &SchemaContext) -> TokenStream {
        match self {
            Root::Leaf(def) => {
                let inner = def.to_impl(ctx);
                let mut tn = quote! {
                    (pub #inner);
                };
                if matches!(def.content, LeafContent::Literal(_)) {
                    let type_ = root_name.to_string();
                    tn.append_all(quote! {
                        impl ::std::str::FromStr for #root_name {
                            type Err = ::xsd::decode::FromXmlError;

                            fn from_str(s: &str) -> Result<Self, Self::Err> {
                                Ok(#root_name(::std::str::FromStr::from_str(s).map_err(|err| {
                                    ::xsd::decode::FromXmlError::ParseType {
                                        type_: #type_.to_string(),
                                        value: s.to_string(),
                                        err: Box::new(err),
                                    }
                                })?))
                            }
                        }

                        impl ::std::fmt::Display for #root_name {
                            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                                self.0.fmt(f)
                            }
                        }
                    })
                }
                tn
            }
            Root::Enum(names) => {
                let names = escape_enum_names(names.clone());
                let variants = names.keys().map(|k| format_ident!("{}", k));
                let from_str_variants = names.iter().map(|(variant, name)| {
                    let name_xml = &name.name;
                    let variant = format_ident!("{}", variant);
                    quote! {
                        #name_xml => #root_name::#variant
                    }
                });
                let as_str_variants = names.iter().map(|(variant, name)| {
                    let name_xml = &name.name;
                    let variant = format_ident!("{}", variant);
                    quote! {
                        #root_name::#variant => #name_xml
                    }
                });

                quote! {
                    {
                        #(#variants,)*
                    }

                    impl ::std::str::FromStr for #root_name {
                        type Err = ::xsd::decode::FromXmlError;

                        fn from_str(s: &str) -> Result<Self, Self::Err> {
                            Ok(match s {
                                #(#from_str_variants,)*
                                _ => return Err(::xsd::decode::FromXmlError::InvalidVariant {
                                    name: s.to_string(),
                                })
                            })
                        }
                    }

                    impl #root_name {
                        pub fn as_str(&self) -> &str {
                            match self {
                                #(#as_str_variants,)*
                            }
                        }
                    }

                    impl ::std::fmt::Display for #root_name {
                        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                            f.write_str(self.as_str())
                        }
                    }
                }
            }
            Root::Element(def) => {
                let inner = def.to_impl(ctx);
                quote! {
                    {
                        #inner
                    }
                }
            }
            Root::Choice(ChoiceDefinition { variants, .. }) => {
                // TODO: use escape_enum_names?
                let variants = variants.iter().map(|variant| {
                    let variant_name = format_ident!("{}", variant.name.name.to_pascal_case());
                    let type_name = variant.definition.to_impl(ctx);
                    quote! {
                        #variant_name(#type_name)
                    }
                });

                quote! {
                    {
                        #(#variants,)*
                    }
                }
            }
        }
    }

    pub fn to_xml_impl(&self, ctx: &SchemaContext) -> TokenStream {
        match self {
            Root::Leaf(def) => {
                let inner = def.to_xml_impl(ctx);
                let tn = quote! {
                    let val = &self.0;
                    #inner
                };
                // TODO: improve those cases to make them easier to understand
                if !matches!(
                    def,
                    LeafDefinition {
                        content: LeafContent::Named(_),
                        ..
                    }
                ) {
                    quote! {
                        ctx.write_start_element(writer)?;
                        #tn
                        ctx.write_end_element(writer)?;
                    }
                } else {
                    tn
                }
            }
            Root::Enum(_) => {
                quote! {
                    ctx.write_start_element(writer)?;
                    let val = self.to_string();
                    if !val.is_empty() {
                        writer.write(XmlEvent::characters(&val))?;
                    }
                    ctx.write_end_element(writer)?;
                }
            }
            Root::Element(def) => def.to_xml_impl(ctx),
            Root::Choice(ChoiceDefinition {
                variants,
                is_virtual,
                ..
            }) => {
                // TODO: use escape_enum_names?
                let variants = variants.iter().map(|variant| {
                    let variant_name = format_ident!("{}", variant.name.name.to_pascal_case());
                    let name_xml = &variant.name.name;
                    let inner = variant.definition.to_xml_impl(ctx);
                    let is_literal = matches!(variant.definition.content, LeafContent::Literal(_));
                    let inner = if is_literal {
                        quote! {
                            ctx.write_start_element(writer)?;
                            #inner
                            ctx.write_end_element(writer)?;
                        }
                    } else {
                        inner
                    };
                    quote! {
                        Self::#variant_name(val) => {
                            let mut ctx = ::xsd::Context::new(#name_xml);
                            #inner
                        }
                    }
                });

                let tn = quote! {
                    match self {
                        #(#variants,)*
                    }
                };

                if *is_virtual {
                    tn
                } else {
                    quote! {
                        ctx.write_start_element(writer)?;
                        #tn
                        ctx.write_end_element(writer)?;
                    }
                }
            }
        }
    }

    pub fn from_xml_impl(&self, name: &Ident, ctx: &SchemaContext) -> TokenStream {
        match self {
            Root::Leaf(def) => {
                let inner = def.from_xml_impl(ctx);
                quote! {
                    #name(#inner)
                }
            }
            Root::Enum(_) => {
                quote! {
                    {
                        let val = node.text()?;
                        ::std::str::FromStr::from_str(val)?
                    }
                }
            }
            Root::Element(def) => {
                let inner = def.from_xml_impl(ctx);
                quote! {
                    #name #inner
                }
            }
            Root::Choice(ChoiceDefinition { variants, .. }) => {
                // TODO: use escape_enum_names?
                let variants = variants.iter().map(|variant| {
                    let variant_name = format_ident!("{}", variant.name.name.to_pascal_case());
                    let inner = variant.definition.from_xml_impl(ctx);
                    let name_xml = &variant.name.name;
                    let namespace_xml = ctx.quote_xml_namespace(&variant.name);
                    if variant.is_virtual {
                        if let LeafContent::Named(name) = &variant.definition.content {
                            let first_name = format_ident!("{}", name.name.to_pascal_case());
                            quote! {
                                if #first_name::lookahead(node) {
                                    Self::#variant_name(#inner)
                                }
                            }
                        } else {
                            // unreachable  ...
                            // TODO: reflect that in the type?
                            unreachable!()
                        }
                    } else {
                        quote! {
                            if let Some(node) = node.next_child(#name_xml, #namespace_xml) {
                                Self::#variant_name(#inner)
                            }
                        }
                    }
                });

                let variant_name = name.to_string();
                quote! {
                    #(#variants else )* {
                        return Err(::xsd::decode::FromXmlError::MissingVariant {
                            name: #variant_name.to_string(),
                        }.into())
                    }
                }
            }
        }
    }

    pub fn lookahead_impl(&self, ctx: &SchemaContext) -> TokenStream {
        match self {
            Root::Leaf(_) => {
                quote! {
                    true
                }
            }
            Root::Enum(_) => {
                quote! {
                    true
                }
            }
            Root::Element(def) => {
                if let ElementDefinition {
                    content: Some(ElementContent::Leaves(leaves)),
                    ..
                } = &def
                {
                    let checks = leaves.iter().scan(false, |prev_required, leaf| {
                        if *prev_required {
                            return None;
                        }
                        *prev_required = !leaf.is_optional();

                        if leaf.is_virtual {
                            let name = if let LeafContent::Named(name) = &leaf.definition.content {
                                name
                            } else {
                                // unreachable  ...
                                // TODO: reflect that in the type?
                                unreachable!()
                            };
                            let name = format_ident!("{}", name.name.to_pascal_case());
                            Some(quote! {
                                #name::lookahead(node)
                            })
                        } else {
                            let name_xml = &leaf.name.name;
                            let namespace_xml = ctx.quote_xml_namespace(&leaf.name);
                            Some(quote! {
                                node.peek_child(#name_xml, #namespace_xml)
                            })
                        }
                    });

                    quote! {
                        false #(|| #checks)*
                    }
                } else {
                    quote! {
                        false
                    }
                }
            }
            Root::Choice(ChoiceDefinition { variants, .. }) => {
                let checks = variants.iter().map(|variant| {
                    if variant.is_virtual {
                        let name = if let LeafContent::Named(name) = &variant.definition.content {
                            name
                        } else {
                            // unreachable  ...
                            // TODO: reflect that in the type?
                            unreachable!()
                        };
                        let name = format_ident!("{}", name.name.to_pascal_case());
                        quote! {
                            #name::lookahead(node)
                        }
                    } else {
                        let name_xml = &variant.name.name;
                        let namespace_xml = ctx.quote_xml_namespace(&variant.name);
                        quote! {
                            node.peek_child(#name_xml, #namespace_xml)
                        }
                    }
                });

                quote! {
                    false #(|| #checks)*
                }
            }
        }
    }
}

fn escape_enum_names(names: Vec<Name>) -> HashMap<String, Name> {
    let mut unknown_count = 0;
    let mut enum_names = HashMap::with_capacity(names.len());

    for name in names.into_iter() {
        let mut variant_name = name
            .name
            .chars()
            .filter_map(|c| match c {
                '_' | '-' => Some('_'),
                c => {
                    if c.is_alphanumeric() {
                        Some(c)
                    } else {
                        None
                    }
                }
            })
            .collect::<String>()
            .to_pascal_case();
        if !variant_name.is_empty()
            && variant_name
                .chars()
                .next()
                .map(|c| !c.is_alphabetic())
                .unwrap_or(false)
        {
            variant_name = format!("V{}", variant_name);
        }

        loop {
            if variant_name.is_empty() || enum_names.contains_key(&variant_name) {
                unknown_count += 1;
                variant_name = format!("Variant{}", unknown_count);
            } else {
                break;
            }
        }

        enum_names.insert(variant_name, name);
    }

    enum_names
}
