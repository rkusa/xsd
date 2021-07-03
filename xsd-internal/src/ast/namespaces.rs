use std::collections::HashMap;

use super::Namespace;

#[derive(Debug)]
pub struct NamespaceDefinition {
    pub namespace: String,
    pub prefix: String,
}

#[derive(Debug, Default)]
pub struct Namespaces {
    by_namespace: HashMap<String, usize>,
    by_id: HashMap<usize, NamespaceDefinition>,
}

impl Namespaces {
    pub fn get_or_insert(&mut self, namespace: &str) -> Namespace {
        let id = if let Some(id) = self.by_namespace.get(namespace) {
            *id
        } else {
            let id = self.by_namespace.len() + 1;
            self.by_namespace.insert(namespace.to_string(), id);
            self.by_id.insert(
                id,
                NamespaceDefinition {
                    namespace: namespace.to_string(),
                    prefix: format!("ns{}", id),
                },
            );
            id
        };
        Namespace::Id(id)
    }

    pub fn get_by_id(&self, id: usize) -> &NamespaceDefinition {
        self.by_id
            .get(&id)
            .expect("inconsistent namespace state, namespace for id is missing")
    }

    pub fn iter(&self) -> impl Iterator<Item = (usize, &NamespaceDefinition)> {
        self.by_id.iter().map(|(id, def)| (*id, def))
    }
}
