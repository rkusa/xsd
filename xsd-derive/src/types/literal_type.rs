use super::{ElementDefault, FromXmlImpl, Namespaces, State, ToImpl, ToXmlImpl};
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug, PartialEq, Clone)]
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
            Any => "any",
        }
    }
}

impl ToImpl for LiteralType {
    fn to_impl(&self, _state: &mut State) -> TokenStream {
        use LiteralType::*;
        match self {
            String => quote! { String },
            Boolean => quote! { bool },
            Int64 => quote! { i64 },
            Uint64 => quote! { u64 },
            Int32 => quote! { i32 },
            Decimal => quote! { rust_decimal::Decimal },
            Float32 => quote! { f32 },
            DateTime => quote! { String },
            Time => quote! { String },
            Date => quote! { String },
            Duration => quote! { String },
            Base64Binary => quote! { String },
            Any => quote! { () },
        }
    }
}

impl ToXmlImpl for LiteralType {
    fn to_xml_impl(&self, _element_default: &ElementDefault) -> TokenStream {
        quote! {
            let val = val.to_string();
            writer.write(XmlEvent::characters(&val))?;
        }
    }
}

impl FromXmlImpl for LiteralType {
    fn from_xml_impl<'a>(
        &self,
        _element_default: &ElementDefault,
        _namespaces: &'a Namespaces<'a>,
    ) -> TokenStream {
        let type_ = self.xsd_name();
        quote! {
            {
                let val = node.text()?;
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
}
