use std::io::{BufReader, Read};
use thiserror::Error;
use xml::attribute::OwnedAttribute;
use xml::EventReader;
use xml::name::OwnedName;
use xml::namespace::Namespace;
use xml::reader::XmlEvent;
use crate::link_extractor::find_links;

#[derive(Error, Debug)]
pub enum TextFileExtractionError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub struct XLinkLink {
    href: String,
    location: String,
    kind: String
}

static xlink_namespace: &str = "http://www.w3.org/1999/xlink";

pub fn extract_links(bytes: &[u8]) -> Result<Vec<XLinkLink>, TextFileExtractionError> {
    let mut collector: Vec<XLinkLink> = vec![];
    
    let parser = EventReader::new(bytes);
    for e in parser {
        let event = e?;
        let event_links: Vec<XLinkLink> = match event {
            XmlEvent::StartElement { name, attributes, namespace } => { from_start_element(&name, &attributes, &namespace) }
            XmlEvent::Comment(_) => {}
            XmlEvent::Characters(_) => {}
            XmlEvent::Whitespace(_) => {}
            _ => {}
        }
        if let XmlEvent::StartElement { name: _, attributes, .. } = event {
            let attributes_with_potential_links = attributes.iter().filter(|att| &att.name.local_name != "Type");
            for attribute in attributes_with_potential_links {
                find_links(&attribute.value).iter().for_each(|link| collector.push(link.to_string()))
            }
        }
    }
}

fn from_start_element(name: &OwnedName, attributes: &Vec<OwnedAttribute>, namespace: &Namespace) -> Vec<XLinkLink> {
    let href = attributes.iter()
        .find(|attribute|
            attribute.name.local_name == "href" && attribute.name.namespace == Some(xlink_namespace.to_string())
        );
    
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_XML: &[u8] = include_bytes!("../../test_files/xml/test.xml");
    #[test]
    fn get_some_website() {
        let links = extract_links(TEST_XML).unwrap();
        assert_eq!(links, vec!["http://www.placeholder-name-here.com/schema/"])
    }
}
