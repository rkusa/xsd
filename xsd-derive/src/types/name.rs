#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Namespace {
    None,
    Target,
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Name {
    pub name: String,
    pub namespace: Namespace,
}

impl Name {
    pub fn new(name: impl Into<String>, namespace: Namespace) -> Self {
        Name {
            name: name.into(),
            namespace,
        }
    }
}
