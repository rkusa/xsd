use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LiteralType {
    String,
    Boolean,
    Int64,
    Uint64,
    Int32,
    Uint32,
    Int16,
    Uint16,
    Int8,
    Uint8,
    Decimal,
    Float32,
    Float64,
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
            Uint32 => "unsignedInt",
            Int16 => "short",
            Uint16 => "unsignedShort",
            Int8 => "byte",
            Uint8 => "unsignedByte",
            Decimal => "decimal",
            Float32 => "float",
            Float64 => "double",
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
            Uint32 => quote! { u32 },
            Int16 => quote! { i16 },
            Uint16 => quote! { u16 },
            Int8 => quote! { i8 },
            Uint8 => quote! { u8 },
            Decimal => quote! { rust_decimal::Decimal },
            Float32 => quote! { f32 },
            Float64 => quote! { f64 },
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
        match self {
            // XSD booleans also allow `1` and `0`, which aren't supported by Rust's `FromStr`
            LiteralType::Boolean => {
                return quote! {
                    match val.trim() {
                        "true" | "1" => true,
                        "false" | "0" => false,
                        _ => return Err(::xsd::decode::FromXmlError::ParseType {
                            type_: #type_.to_string(),
                            value: val.to_string(),
                            err: Box::new(<bool as std::str::FromStr>::from_str(val).unwrap_err()),
                        }.into()),
                    }
                };
            }
            // all non-string types have a whitespace facet of `collapse`, so ignore any
            // surrounding whitespace
            LiteralType::String | LiteralType::Any => {}
            _ => {
                return quote! {
                    std::str::FromStr::from_str(val.trim()).map_err(|err| {
                        ::xsd::decode::FromXmlError::ParseType {
                            type_: #type_.to_string(),
                            value: val.to_string(),
                            err: Box::new(err),
                        }
                    })?
                };
            }
        }
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
