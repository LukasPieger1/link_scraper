use thiserror::Error;
use xml::attribute::OwnedAttribute;
use xml::common::{Position, TextPosition};
use xml::EventReader;
use xml::name::OwnedName;
use xml::namespace::Namespace;
use xml::reader::XmlEvent;
use crate::link_extractor::find_links;

#[derive(Error, Debug)]
pub enum XmlError {
    #[error(transparent)]
    XmlReaderError(#[from] xml::reader::Error)
}

#[cfg(feature = "xlink")]
pub mod xlink;

#[derive(Debug, Copy, Clone)]
pub enum XmlLinkType {
    Href
}

#[derive(Debug)]
pub struct XmlLink {
    pub href: String,
    pub location: TextPosition,
    pub kind: XmlLinkType
}

pub struct XmlStartElement<'a> {
    name: &'a OwnedName,
    attributes: &'a Vec<OwnedAttribute>,
    _namespace: &'a Namespace
}

/// Extracts all links from href-attributes regardless of their namespace or tag-name
pub fn extract_links_from_href_tags(bytes: &[u8]) -> Result<Vec<XmlLink>, XmlError> {
    let mut collector: Vec<XmlLink> = vec![];

    let mut parser = EventReader::new(bytes);
    while let Ok(xml_event) = &parser.next() {
        match xml_event {
            XmlEvent::StartElement { name: _name, attributes, namespace: _namespace } => {
                let mut list = attributes.iter()
                    .filter_map(|attribute| {
                        if &attribute.name.local_name != "href" {
                            return None
                        }
                        return Some(find_links(&attribute.value))
                    })
                    .flatten()
                    .map(|link| XmlLink { href: link.to_string(), location: parser.position(), kind: XmlLinkType::Href })
                    .collect();
                collector.append(&mut list)
            }
            XmlEvent::EndDocument => break,
            _ => {}
        }
    }

    Ok(collector)
}

pub fn extract_links(bytes: &[u8]) -> Result<Vec<XmlLink>, XmlError> {
    let mut collector: Vec<XmlLink> = vec![];
    
    let mut parser = EventReader::new(bytes);
    while let Ok(xml_event) = &parser.next() {
        match xml_event {
            XmlEvent::StartDocument { .. } => {}
            XmlEvent::EndDocument => {}
            XmlEvent::ProcessingInstruction { .. } => {}
            XmlEvent::StartElement { name, attributes, namespace } => {
                let start_element = XmlStartElement {name: &name, attributes: &attributes, _namespace: &namespace};
                let mut links = from_xml_start_element(&start_element)?;
                collector.append(&mut links)
            }
            XmlEvent::EndElement { .. } => {}
            XmlEvent::CData(_) => {}
            XmlEvent::Comment(_) => {}
            XmlEvent::Characters(_) => {}
            XmlEvent::Whitespace(_) => {}
        }
    }
    
    Ok(collector)
}

fn from_xml_start_element(start_element: &XmlStartElement) -> Result<Vec<XmlLink>, XmlError> {
    start_element
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_XLINK: &[u8] = include_bytes!("../../../test_files/xml/xlink_test.xml");

    #[test]
    fn extract_hrefs() {
        let links = extract_links_from_href_tags(TEST_XLINK).unwrap();
        println!("{:?}", links);
        assert_eq!(1, links.len())
    }
}
