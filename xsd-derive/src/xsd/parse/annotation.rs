use crate::xsd::context::NS_XSD;
use crate::xsd::node::Node;
use crate::xsd::XsdError;

pub fn parse(node: &Node<'_, '_>) -> Result<Option<String>, XsdError> {
    let docs =
        node.children()
            .namespace(NS_XSD)
            .iter()
            .try_fold(None, |docs, child| match child.name() {
                "documentation" => {
                    if docs.is_some() {
                        // TODO: prefer documentation with xml:lang="en"
                        Ok(docs)
                    } else {
                        let text = child.try_text()?;
                        if !text.is_empty() {
                            Ok(Some(cleanup_docs(&text)))
                        } else {
                            Ok(None)
                        }
                    }
                }
                child_name => Err(XsdError::UnsupportedElement {
                    name: child_name.to_string(),
                    parent: node.name().to_string(),
                    range: child.range(),
                }),
            })?;

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
