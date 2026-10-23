use std::collections::HashSet;

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
                let variants = names.iter().map(|(k, _)| format_ident!("{}", k));
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
                let variant_names = choice_variant_idents(variants);
                let variants = variants
                    .iter()
                    .zip(variant_names)
                    .map(|(variant, variant_name)| {
                        let mut type_ident = variant.definition.to_impl(ctx);
                        if variant.is_vec() {
                            type_ident = quote! { Vec<#type_ident> }
                        }
                        quote! {
                            #variant_name(#type_ident)
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
                let variant_names = choice_variant_idents(variants);
                let variants = variants
                    .iter()
                    .zip(variant_names)
                    .map(|(variant, variant_name)| {
                        let name_xml = ctx.get_xml_element_name(&variant.name);
                        let inner = variant.definition.to_xml_impl(ctx);
                        let is_literal =
                            matches!(variant.definition.content, LeafContent::Literal(_));
                        let mut inner = if is_literal {
                            quote! {
                                let mut ctx = ::xsd::Context::new(#name_xml);
                                ctx.write_start_element(writer)?;
                                #inner
                                ctx.write_end_element(writer)?;
                            }
                        } else {
                            quote! {
                                let mut ctx = ::xsd::Context::new(#name_xml);
                                #inner
                            }
                        };
                        if variant.is_vec() {
                            inner = quote! {
                                for val in val {
                                    #inner
                                }
                            };
                        }

                        quote! {
                            Self::#variant_name(val) => {
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
                let variant_names = choice_variant_idents(variants);
                let variants = variants.iter().zip(variant_names).map(|(variant, variant_name)| {
                    let inner = variant.definition.from_xml_impl(ctx);
                    let name_xml = &variant.name.name;
                    let namespace_xml = ctx.quote_xml_namespace(&variant.name);
                    if variant.is_virtual {
                        if let LeafContent::Named(name) = &variant.definition.content {
                            let first_name = ctx.type_ident(name);
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
                        #[allow(clippy::collapsible_else_if)]
                        if variant.is_vec() {
                            quote! {
                                if node.peek_child(#name_xml, #namespace_xml) {
                                    let mut vec = Vec::new();
                                    while let Some(node) = node.next_child(#name_xml, #namespace_xml) {
                                        vec.push(#inner);
                                    }
                                    Self::#variant_name(vec)
                                }
                            }
                        } else {
                            quote! {
                                if let Some(node) = node.next_child(#name_xml, #namespace_xml) {
                                    Self::#variant_name(#inner)
                                }
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
                            let name = ctx.type_ident(name);
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
                        let name = ctx.type_ident(name);
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

/// Derives valid and unique Rust identifiers for the given enum variants (preserving the order
/// in which they are defined in the schema).
fn escape_enum_names(names: Vec<Name>) -> Vec<(String, Name)> {
    let variants = escape_variant_names(names.iter().map(|name| name.name.as_str()));
    variants.into_iter().zip(names).collect()
}

/// Derives valid and unique Rust identifiers for the variants of the given choice.
fn choice_variant_idents(variants: &[Leaf]) -> Vec<Ident> {
    escape_variant_names(variants.iter().map(|variant| variant.name.name.as_str()))
        .into_iter()
        .map(|name| format_ident!("{}", name))
        .collect()
}

/// Derives valid and unique Rust identifiers for the given variant names (preserving their
/// order).
fn escape_variant_names<'a>(names: impl IntoIterator<Item = &'a str>) -> Vec<String> {
    let mut unknown_count = 0;
    let mut used = HashSet::new();
    let mut variants = Vec::new();

    for name in names {
        let mut variant_name = name
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
        // the only keyword that is in pascal case
        if variant_name == "Self" {
            variant_name = "Self_".to_string();
        }

        if variant_name.is_empty() {
            loop {
                unknown_count += 1;
                variant_name = format!("Variant{}", unknown_count);
                if !used.contains(&variant_name) {
                    break;
                }
            }
        } else if used.contains(&variant_name) {
            let mut n = 2;
            while used.contains(&format!("{variant_name}{n}")) {
                n += 1;
            }
            variant_name = format!("{variant_name}{n}");
        }

        used.insert(variant_name.clone());
        variants.push(variant_name);
    }

    variants
}

#[cfg(test)]
mod tests {
    use super::escape_variant_names;

    #[test]
    fn escape_variants() {
        assert_eq!(
            escape_variant_names(["foo-bar", "1st", "self", "", "fooBar", "a.b"]),
            vec!["FooBar", "V1St", "Self_", "Variant1", "FooBar2", "Ab"]
        );
    }
}
