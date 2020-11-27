mod annotation;
mod attribute;
mod choice;
mod complex_content;
pub mod complex_type;
pub mod element;
mod sequence;
mod simple_content;
pub mod simple_type;

use super::context::Context;
use crate::ast::Name;

fn derive_virtual_name<'a, 'input, 'b>(
    names: impl IntoIterator<Item = &'b Name>,
    ctx: &'b Context<'a, 'input>,
) -> Name
where
    'a: 'input,
{
    let mut virtual_name = String::new();
    for name in names.into_iter() {
        if !name.name.is_empty() {
            virtual_name += (&name.name[0..1]).to_ascii_uppercase().as_str();
            virtual_name += &name.name[1..];
        }
    }

    ctx.get_node_name(&virtual_name, false)
}
