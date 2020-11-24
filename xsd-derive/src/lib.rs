// #![allow(unused)]

mod ast;
mod error;
mod generator;
mod xsd;

use std::env;
use std::path::PathBuf;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn all(args: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemMod);
    let args = syn::parse_macro_input!(args as syn::AttributeArgs);

    // TODO: restrict to only one element attribute
    // TODO: validate provided arguments

    generate(input, args).unwrap_or_else(|e| e.to_compile_error().into())
}

fn generate(input: syn::ItemMod, args: syn::AttributeArgs) -> Result<TokenStream, syn::Error> {
    // let sig = &input.sig;
    // let attrs = &input.attrs;
    // let vis = input.vis;

    // TODO: make sure that the attribute was added to an empty struct

    // if !input.fields.is_empty() {
    //     return Err(syn::Error::new_spanned(
    //         input.fields,
    //         "XSD element struct must not have fields",
    //     ));
    // }
    // if !input.generics.params.is_empty() {
    //     return Err(syn::Error::new_spanned(
    //         input.fields,
    //         "XSD element struct must not have generics",
    //     ));
    // }

    let schema_path = {
        let mut schema_path = None;

        for arg in args {
            match arg {
                syn::NestedMeta::Meta(syn::Meta::NameValue(nv)) => {
                    let path = if let syn::Lit::Str(path) = nv.lit {
                        path.value()
                    } else {
                        return Err(syn::Error::new_spanned(nv.lit, "Expected path"));
                    };
                    if nv.path.is_ident("schema") {
                        schema_path = Some(path);
                    } else {
                        return Err(syn::Error::new_spanned(nv.path, "Unknown attribute name"));
                    }
                }
                other => return Err(syn::Error::new_spanned(other, "Unsupported attribute")),
            }
        }

        schema_path.ok_or_else(|| syn::Error::new_spanned(&input, "Argument `schema` required"))?
    };

    let mut path = PathBuf::from(
        env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR env to be defined"),
    );
    path.push(&schema_path);
    let generated_code = match generator::generate(&input, path) {
        Ok(result) => result,
        Err(err) => return Err(syn::Error::new_spanned(input, err)),
    };

    Ok(generated_code.into())
}
