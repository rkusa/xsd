use std::path::Path;

use proc_macro2::TokenStream;
use quote::quote;
use xsd_internal::xsd::schema::{Schema, SchemaError};

pub fn generate(item: &syn::ItemMod, path: impl AsRef<Path>) -> Result<TokenStream, SchemaError> {
    let schema = Schema::parse_file(path)?;
    let structs = schema.generate_all()?;

    let attrs = &item.attrs;
    let vis = &item.vis;
    let ident = &item.ident;
    let items = item
        .content
        .as_ref()
        .map(|(_, items)| items.clone())
        .unwrap_or_default();

    let result = quote! {
        #(#attrs)*
        #vis mod #ident {
            #(#items
            )*

            #structs
        }
    };

    // eprintln!("{}", result.to_string());
    Ok(result)
}
