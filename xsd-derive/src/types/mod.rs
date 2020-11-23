mod element;
mod element_content;
mod element_definition;
mod literal;
mod literal_type;
mod name;

use std::collections::HashMap;

pub use element::*;
pub use element_content::*;
pub use element_definition::*;
pub use literal::*;
pub use literal_type::*;
pub use name::*;

use proc_macro2::TokenStream;

pub trait ToImpl {
    fn to_impl(&self, state: &mut State) -> TokenStream;
}

pub type State = ();

#[derive(Clone)]
pub enum ElementDefault {
    Qualified,
    Unqualified(String),
}

pub trait ToXmlImpl {
    fn to_xml_impl(&self, element_default: &ElementDefault) -> TokenStream;
}

type Namespaces<'a> = HashMap<&'a str, &'a str>;

pub trait FromXmlImpl {
    fn from_xml_impl<'a>(
        &self,
        element_default: &ElementDefault,
        namespaces: &'a Namespaces<'a>,
    ) -> TokenStream;
}

impl ElementDefault {
    pub fn get_xml_name(&self, name: &Name) -> String {
        match &name.namespace {
            Namespace::None => name.name.clone(),
            Namespace::Target => {
                match self {
                    ElementDefault::Qualified => {
                        unimplemented!("ElementDefault::get_xml_name Namespace::Target");
                    }
                    ElementDefault::Unqualified(_) => {
                        // TODO: whether namespaces matches default target
                        name.name.clone()
                    }
                }
            }
            Namespace::Other(other) => {
                unimplemented!("ElementDefault::get_xml_name Namespace::Other({})", other)
            }
        }
        // match self {
        //     ElementDefault::Qualified => format!("{}:{}", type_name.prefix, type_name.name),
        //     ElementDefault::Unqualified(prefix) => {
        //         if prefix == &type_name.prefix {
        //             type_name.name.to_string()
        //         } else {
        //             format!("{}:{}", type_name.prefix, type_name.name)
        //         }
        //     }
        // }
    }
}
