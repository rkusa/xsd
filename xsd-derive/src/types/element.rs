use super::{ElementDefinition, Name};

#[derive(Debug, Clone)]
pub struct Element {
    pub name: Name,
    pub definition: ElementDefinition,
}
