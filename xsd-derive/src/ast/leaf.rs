use super::{LeafDefinition, Name};

#[derive(Debug, Clone)]
pub struct Leaf {
    pub name: Name,
    pub definition: LeafDefinition,
}
