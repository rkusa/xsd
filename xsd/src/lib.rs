pub mod decode;

pub use xml;
pub use xsd_derive::all;
pub use xsd_internal as internal;

use std::borrow::Cow;
use std::mem;

use xml::attribute::Attribute;
use xml::name::Name;
use xml::namespace::{Namespace, NS_NO_PREFIX};
use xml::writer::events::XmlEvent;

pub enum Context<'a, 'b> {
    Virtual(&'b mut ContextInner<'a>),
    Context(ContextInner<'a>),
}

pub struct ContextInner<'a> {
    name: Name<'a>,
    attributes: Vec<(Name<'a>, Cow<'a, str>)>,
    namespace: Namespace,
}

impl<'a, 'b> Context<'a, 'b> {
    pub fn new(name: impl Into<Name<'a>>) -> Self {
        Context::Context(ContextInner {
            name: name.into(),
            attributes: Vec::new(),
            namespace: Namespace::empty(),
        })
    }

    pub fn wrap<'c>(ctx: &'c mut Context<'a, 'b>) -> Context<'a, 'c> {
        match ctx {
            Context::Virtual(inner) => Context::Virtual(inner),
            Context::Context(inner) => Context::Virtual(inner),
        }
    }

    pub fn set_attr(&mut self, name: impl Into<Name<'a>>, value: impl Into<Cow<'a, str>>) {
        match self {
            Context::Virtual(inner) => inner.attributes.push((name.into(), value.into())),
            Context::Context(inner) => inner.attributes.push((name.into(), value.into())),
        }
    }

    pub fn set_ns(&mut self, prefix: impl Into<String>, uri: impl Into<String>) {
        match self {
            Context::Virtual(inner) => {
                inner.namespace.put(prefix, uri);
            }
            Context::Context(inner) => {
                inner.namespace.put(prefix, uri);
            }
        }
    }

    pub fn set_default_ns(&mut self, uri: impl Into<String>) {
        match self {
            Context::Virtual(inner) => {
                inner.namespace.put(NS_NO_PREFIX, uri);
            }
            Context::Context(inner) => {
                inner.namespace.put(NS_NO_PREFIX, uri);
            }
        }
    }

    pub fn write_start_element<W: std::io::Write>(
        &mut self,
        writer: &mut ::xml::writer::EventWriter<W>,
    ) -> Result<(), xml::writer::Error> {
        match self {
            Context::Virtual(_) => {}
            Context::Context(inner) => {
                let attributes = mem::take(&mut inner.attributes);
                let namespace = mem::replace(&mut inner.namespace, Namespace::empty());
                writer.write(XmlEvent::StartElement {
                    name: inner.name,
                    attributes: Cow::Owned(
                        attributes
                            .iter()
                            .map(|(k, v)| Attribute::new(*k, v))
                            .collect(),
                    ),
                    namespace: Cow::Owned(namespace),
                })?
            }
        }

        Ok(())
    }

    pub fn write_end_element<W: std::io::Write>(
        &mut self,
        writer: &mut ::xml::writer::EventWriter<W>,
    ) -> Result<(), xml::writer::Error> {
        match self {
            Context::Virtual(_) => {}
            Context::Context(_) => writer.write(XmlEvent::end_element())?,
        }

        Ok(())
    }
}
