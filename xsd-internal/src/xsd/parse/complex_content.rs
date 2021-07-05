use crate::ast::{
    ElementContent, ElementDefinition, Leaf, LeafContent, LeafDefinition, MaxOccurs, MinOccurs,
    Name, Root,
};
use crate::xsd::context::{Context, NS_XSD};
use crate::xsd::error::XsdError;
use crate::xsd::node::Node;

use super::element::parse_min_occurs;

pub fn parse<'a, 'input>(
    node: Node<'a, 'input>,
    parent: &Name,
    ctx: &mut Context<'input>,
) -> Result<ElementDefinition, XsdError>
where
    'a: 'input,
{
    node.prevent_unvisited_attributes()?;

    let mut children = node.children().namespace(NS_XSD).collect();
    let extension = children.try_remove("extension", Some(NS_XSD))?;
    children.prevent_unvisited_children()?;

    let base = extension.try_attribute("base")?;
    let content = ctx.get_type_name(&base)?;
    let base_name = match &content {
        LeafContent::Literal(_) | LeafContent::Fixed(_) => {
            return Err(XsdError::UnsupportedAttributeValue {
                name: "base".to_string(),
                value: base.value().to_string(),
                element: "extension".to_string(),
                range: base.range(),
            })
        }
        LeafContent::Named(name) => {
            ctx.discover_type(name, Some(parent));
            name
        }
    };

    let mut children = extension.children().namespace(NS_XSD).collect();
    let mut virtual_leaves = Vec::new();

    if let Some(child) = children.remove("sequence", Some(NS_XSD)) {
        let min_occurs = parse_min_occurs(child.attribute("minOccurs"))?;

        let leaves = super::sequence::parse(child, parent, ctx)?;
        let leaf_name = ctx.get_node_name(&base_name.name, false);
        let root_name = super::derive_virtual_name(vec![parent, &leaf_name], ctx, false);

        ctx.add_root(
            root_name.clone(),
            Root::Element(ElementDefinition {
                attributes: Vec::new(),
                content: Some(ElementContent::Leaves(leaves)),
                is_virtual: true,
                docs: None,
            }),
        );
        ctx.discover_type(&root_name, Some(parent));

        virtual_leaves.push(Leaf {
            name: leaf_name,
            definition: LeafDefinition {
                content: LeafContent::Named(root_name),
                restrictions: Vec::new(),
                docs: None,
            },
            is_unordered: false,
            is_virtual: true,
            min_occurs,
            max_occurs: MaxOccurs::default(),
        });
    }

    // read all attributes
    let mut attributes = Vec::new();
    while let Some(child) = children.remove("attribute", Some(NS_XSD)) {
        if let Some(attr) = super::attribute::parse(child, parent, ctx)? {
            attributes.push(attr);
        }
    }

    children.prevent_unvisited_children()?;

    // TODO: merge with extension instead of having it as `value` property?
    Ok(ElementDefinition {
        attributes,
        content: Some(if virtual_leaves.is_empty() {
            ElementContent::Leaf(
                base_name.clone(),
                LeafDefinition {
                    content,
                    restrictions: Vec::new(),
                    docs: None,
                },
            )
        } else {
            let mut leaves = vec![Leaf {
                name: ctx.get_node_name("base", false),
                definition: LeafDefinition {
                    content,
                    restrictions: Vec::new(),
                    docs: None,
                },
                is_unordered: false,
                is_virtual: true,
                min_occurs: MinOccurs::default(),
                max_occurs: MaxOccurs::default(),
            }];
            leaves.extend(virtual_leaves);
            ElementContent::Leaves(leaves)
        }),
        is_virtual: false,
        docs: None,
    })
}
