mod attribute;
mod element_content;
mod element_definition;
mod leaf;
mod leaf_content;
mod leaf_definition;
mod literal_type;
mod name;
mod root;

use std::collections::HashMap;

pub use attribute::*;
pub use element_content::*;
pub use element_definition::*;
pub use leaf::*;
pub use leaf_content::*;
pub use leaf_definition::*;
pub use literal_type::*;
pub use name::*;
pub use root::*;

// TODO: implement or remove
pub struct ElementDefault {
    pub target_namespace: Option<String>,
    pub qualified: bool,
}
pub type State = ();

type Namespaces<'a> = HashMap<&'a str, &'a str>;

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
