use crate::xsd::context::NS_XSD;
use crate::xsd::node::Node;
use crate::xsd::XsdError;

pub fn parse(node: Node<'_, '_>) -> Result<Option<String>, XsdError> {
    let mut children = node.children().namespace(NS_XSD).collect();
    let docs = children
        .remove("documentation", Some(NS_XSD))
        .map(|child| child.try_text())
        .transpose()?
        .and_then(|text| {
            if !text.is_empty() {
                Some(cleanup_docs(&text))
            } else {
                None
            }
        });

    children.prevent_unvisited_children()?;
    node.prevent_unvisited_attributes()?;

    Ok(docs)
}

pub fn cleanup_docs(docs: &str) -> String {
    docs.split_whitespace().collect::<Vec<&str>>().join(" ")
}

#[cfg(test)]
mod test {
    use super::cleanup_docs;

    #[test]
    fn remove_newlines_from_docs() {
        let docs = cleanup_docs("Something\n\t\t\tthat continues on the next line.");
        assert_eq!(docs, "Something that continues on the next line.");
    }
}
