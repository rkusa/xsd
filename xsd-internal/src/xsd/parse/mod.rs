mod annotation;
mod attribute;
mod choice;
mod complex_content;
pub mod complex_type;
pub mod element;
pub mod root;
mod sequence;
mod simple_content;
pub mod simple_type;

use inflector::Inflector;

use super::error::XsdError;
use super::node::Node;
use crate::ast::{Leaf, Name};

/// The name of a field containing a group of the given elements (e.g. a `<xs:choice>` inside of
/// a `<xs:sequence>`). This is either the words all names have in common (e.g. `angle` for
/// `angleDeg` and `angleRad`), both names joined by `joiner` for groups of two (e.g. `aOrB`), or
/// the `fallback` for larger groups.
fn group_name(names: &[&Name], joiner: &str, fallback: &str) -> String {
    if let Some(name) = common_words(names) {
        return name;
    }
    match names {
        [a, b] => format!("{}{}{}", a.name, joiner, upper_first(&b.name)),
        _ => fallback.to_string(),
    }
}

/// The name of a `<xs:sequence>` inside of a `<xs:choice>`. This is either the words all names
/// have in common, or the name of the first element (which is usually what distinguishes the
/// variants of a choice).
fn variant_name(names: &[&Name]) -> String {
    common_words(names)
        .or_else(|| names.first().map(|name| name.name.clone()))
        .unwrap_or_else(|| "sequence".to_string())
}

/// Appends a number to the `name` if it is already used by one of the `leaves`.
fn unique_leaf_name(name: String, leaves: &[Leaf]) -> String {
    let is_used = |name: &str| {
        leaves
            .iter()
            .any(|leaf| leaf.name.name.to_snake_case() == name.to_snake_case())
    };
    if !is_used(&name) {
        return name;
    }
    let mut n = 2;
    while is_used(&format!("{name}{n}")) {
        n += 1;
    }
    format!("{name}{n}")
}

/// The longer one of the leading or trailing words all names have in common (or the name itself
/// if there is only one).
fn common_words(names: &[&Name]) -> Option<String> {
    let words = names
        .iter()
        .map(|name| name.name.to_snake_case())
        .collect::<Vec<_>>();
    let words = words
        .iter()
        .map(|name| name.split('_').collect::<Vec<_>>())
        .collect::<Vec<_>>();
    let (first, rest) = words.split_first()?;

    let prefix = first
        .iter()
        .enumerate()
        .take_while(|(i, word)| rest.iter().all(|other| other.get(*i) == Some(word)))
        .count();
    let suffix = first
        .iter()
        .rev()
        .enumerate()
        .take_while(|(i, word)| {
            rest.iter()
                .all(|other| other.len() > *i && other[other.len() - 1 - i] == **word)
        })
        .count();

    let common = if prefix >= suffix {
        &first[..prefix]
    } else {
        &first[first.len() - suffix..]
    };
    if common.is_empty() {
        None
    } else {
        Some(common.join("_"))
    }
}

fn upper_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

fn parse_annotation(node: Option<Node<'_, '_>>) -> Result<Option<String>, XsdError> {
    Ok(node.map(annotation::parse).transpose()?.flatten())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Namespace;

    fn names(names: &[&str]) -> Vec<Name> {
        names
            .iter()
            .map(|n| Name::new(*n, Namespace::None))
            .collect()
    }

    fn group(n: &[&str]) -> String {
        let names = names(n);
        group_name(&names.iter().collect::<Vec<_>>(), "Or", "choice")
    }

    #[test]
    fn test_group_name() {
        assert_eq!(group(&["AngleDeg", "AngleRad", "AngleSec"]), "angle");
        assert_eq!(group(&["FirstName", "LastName", "FullName"]), "name");
        assert_eq!(group(&["orderDate", "orderedBy"]), "orderDateOrOrderedBy");
        assert_eq!(group(&["user", "bot"]), "userOrBot");
        assert_eq!(group(&["flat", "planoConcave", "biConvex"]), "choice");
        assert_eq!(group(&["point"]), "point");
    }

    #[test]
    fn test_variant_name() {
        let n = names(&["confirmedOrderId", "salesOrderId", "orderStatus"]);
        assert_eq!(
            variant_name(&n.iter().collect::<Vec<_>>()),
            "confirmedOrderId"
        );
        let n = names(&["lengthLeft", "lengthRight"]);
        assert_eq!(variant_name(&n.iter().collect::<Vec<_>>()), "length");
    }
}
