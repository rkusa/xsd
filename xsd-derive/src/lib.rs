// #![allow(unused)]

mod generator;

use std::env;
use std::path::PathBuf;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn all(args: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemMod);

    let mut schema_path: Option<String> = None;
    let args_parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("schema") {
            let value: syn::LitStr = meta.value()?.parse()?;
            schema_path = Some(value.value());
            Ok(())
        } else {
            Err(meta.error("unsupported property"))
        }
    });
    syn::parse_macro_input!(args with args_parser);

    // TODO: restrict to only one element attribute
    // TODO: validate provided arguments

    let Some(schema_path) = schema_path else {
        return syn::Error::new_spanned(&input, "Argument `schema` required")
            .to_compile_error()
            .into();
    };
    generate(input, schema_path).unwrap_or_else(|e| e.to_compile_error().into())
}

fn generate(input: syn::ItemMod, schema_path: String) -> Result<TokenStream, syn::Error> {
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
