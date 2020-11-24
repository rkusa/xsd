use super::{LeafContent, Name};

#[derive(Debug, Clone)]
pub struct Leaf {
    pub name: Name,
    pub content: LeafContent,
    // pub docs: Option<String>,
    // pub restrictions: Option<Vec<Restriction>>,
    // pub fixed: Option<String>,
}
