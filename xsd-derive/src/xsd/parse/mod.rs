mod annotation;
mod attribute;
mod choice;
mod complex_content;
pub mod complex_type;
pub mod element;
mod sequence;
mod simple_content;
pub mod simple_type;

use inflector::Inflector;

use super::context::Context;
use crate::ast::Name;

fn derive_virtual_name<'a, 'input, 'b>(
    names: impl IntoIterator<Item = &'b Name> + Clone,
    ctx: &'b Context<'a, 'input>,
    shorten: bool,
) -> Name
where
    'a: 'input,
{
    if shorten {
        let mprefix = derive_mutal_prefix(names.clone().into_iter().map(|n| n.name.as_str()));
        let msuffix = derive_mutal_suffix(names.clone().into_iter().map(|n| n.name.as_str()));

        match (mprefix, msuffix) {
            (Some(prefix), Some(suffix)) => {
                return ctx.get_node_name(
                    if prefix.len() > suffix.len() {
                        &prefix
                    } else {
                        &suffix
                    },
                    false,
                )
            }
            (Some(prefix), None) => return ctx.get_node_name(&prefix, false),
            (None, Some(suffix)) => return ctx.get_node_name(&suffix, false),
            _ => {}
        }
    }

    let mut virtual_name = String::new();
    for name in names.into_iter() {
        if !name.name.is_empty() {
            virtual_name += (&name.name[0..1]).to_ascii_uppercase().as_str();
            virtual_name += &name.name[1..];
        }
    }

    ctx.get_node_name(&virtual_name, false)
}

fn derive_mutal_prefix<'a>(names: impl IntoIterator<Item = &'a str>) -> Option<String> {
    let mut names = names.into_iter();
    if let Some(first) = names.next() {
        let name = names.fold(first.to_string(), |lhs, rhs| mutal_prefix(&lhs, rhs));
        if name.is_empty() {
            None
        } else {
            Some(name)
        }
    } else {
        None
    }
}

fn mutal_prefix(lhs: &str, rhs: &str) -> String {
    lhs.to_snake_case()
        .chars()
        .zip(rhs.to_snake_case().chars())
        .take_while(|(a, b)| a == b && *a != '_' && *b != '_')
        .map(|(a, _)| a)
        .collect()
}

fn derive_mutal_suffix<'a>(names: impl IntoIterator<Item = &'a str>) -> Option<String> {
    let mut names = names.into_iter();
    if let Some(first) = names.next() {
        let name = names.fold(first.to_string(), |lhs, rhs| mutal_suffix(&lhs, rhs));
        if name.is_empty() {
            None
        } else {
            Some(name)
        }
    } else {
        None
    }
}

fn mutal_suffix(lhs: &str, rhs: &str) -> String {
    lhs.to_snake_case()
        .chars()
        .rev()
        .zip(rhs.to_snake_case().chars().rev())
        .take_while(|(a, b)| a == b && *a != '_' && *b != '_')
        .map(|(a, _)| a)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mutual_prefix() {
        assert_eq!(mutal_prefix("AngleDeg", "AngleRad"), "angle".to_string());
    }

    #[test]
    fn test_mutual_suffix() {
        assert_eq!(mutal_suffix("FirstName", "LastName"), "name".to_string());
    }

    #[test]
    fn test_derive_mutal_prefix() {
        assert_eq!(
            derive_mutal_prefix(vec!["AngleDeg", "AngleRad", "AngleSec"]),
            Some("angle".to_string())
        );
    }

    #[test]
    fn test_derive_mutal_suffix() {
        assert_eq!(
            derive_mutal_suffix(vec!["FirstName", "LastName", "FullName"]),
            Some("name".to_string())
        );
    }
}
