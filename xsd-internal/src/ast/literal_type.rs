use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LiteralType {
    String,
    Boolean,
    Int64,
    Uint64,
    Int32,
    Decimal,
    Float32,
    DateTime,
    Time,
    Date,
    Duration,
    Base64Binary,
    HexBinary,
    Any,
}

impl LiteralType {
    pub fn xsd_name(&self) -> &str {
        use LiteralType::*;
        match self {
            String => "string",
            Boolean => "boolean",
            Int64 => "long",
            Uint64 => "nonNegativeInteger",
            Int32 => "int",
            Decimal => "decimal",
            Float32 => "float",
            DateTime => "dateTime",
            Time => "time",
            Date => "date",
            Duration => "duration",
            Base64Binary => "base64Binary",
            HexBinary => "hexBinary",
            Any => "any",
        }
    }
}

impl LiteralType {
    pub fn to_impl(&self) -> TokenStream {
        use LiteralType::*;
        match self {
            String => quote! { String },
            Boolean => quote! { bool },
            Int64 => quote! { i64 },
            Uint64 => quote! { u64 },
            Int32 => quote! { i32 },
            Decimal => quote! { rust_decimal::Decimal },
            Float32 => quote! { f32 },
            Any => quote! { String },

            // TODO: use proper types for the following
            DateTime => quote! { String },
            Time => quote! { String },
            Date => quote! { String },
            Duration => quote! { String },
            Base64Binary => quote! { String },
            HexBinary => quote! { String },
        }
    }

    pub fn to_xml_impl(&self) -> TokenStream {
        quote!(val.to_string())
    }

    pub fn from_str_impl(&self) -> TokenStream {
        let type_ = self.xsd_name();
        quote! {
            std::str::FromStr::from_str(val).map_err(|err| {
                ::xsd::decode::FromXmlError::ParseType {
                    type_: #type_.to_string(),
                    value: val.to_string(),
                    err: Box::new(err),
                }
            })?
        }
    }
}
