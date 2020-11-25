mod attribute;
mod element_content;
mod element_definition;
mod leaf;
mod leaf_content;
mod literal_type;
mod name;

use std::collections::HashMap;

pub use attribute::*;
pub use element_content::*;
pub use element_definition::*;
pub use leaf::*;
pub use leaf_content::*;
pub use literal_type::*;
pub use name::*;

use proc_macro2::TokenStream;

pub trait ToImpl {
    fn to_impl(&self, state: &mut State) -> TokenStream;
}

// TODO: implement or remove
pub struct ElementDefault {
    pub target_namespace: Option<String>,
    pub qualified: bool,
}
pub type State = ();

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

pub fn get_xml_name(name: &Name, qualified: bool) -> String {
    match &name.namespace {
        Namespace::None => name.name.clone(),
        Namespace::Target => {
            if qualified {
                name.name.clone()
            } else {
                format!("tn:{}", name.name)
            }
        }
        Namespace::Other(other) => {
            unimplemented!("ElementDefault::get_xml_name Namespace::Other({})", other)
        }
    }
}
