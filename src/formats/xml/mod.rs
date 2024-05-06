use thiserror::Error;
use xml::attribute::OwnedAttribute;
use xml::common::{Position, TextPosition};
use xml::EventReader;
use xml::name::OwnedName;
use xml::namespace::Namespace;
use xml::reader::XmlEvent;
use crate::link_extractor::find_urls;

#[derive(Error, Debug)]
pub enum XmlError {
    #[error(transparent)]
    XmlReaderError(#[from] xml::reader::Error)
}

#[cfg(feature = "xlink")]
pub mod xlink;

#[derive(Debug, Clone)]
pub enum XmlLinkType {
    Attribute(OwnedAttribute),
    Comment,
    PlainText,
}

#[derive(Debug)]
pub struct XmlLink {
    pub url: String,
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
                let mut list: Vec<XmlLink> = from_xml_start_element_attributes(attributes, &parser)?
                    .into_iter()
                    .filter(|link| {
                    if let XmlLinkType::Attribute(att) = &link.kind {
                        if att.name.local_name == "href" {
                            return true
                        }
                    }
                    return false
                }).collect();
                collector.append(&mut list)
            }
            XmlEvent::EndDocument => break,
            _ => {}
        }
    }

    Ok(collector)
}

/// Extracts links from any file with a xml-schema
pub fn extract_links(bytes: &[u8]) -> Result<Vec<XmlLink>, XmlError> {
    let mut collector: Vec<XmlLink> = vec![];
    
    let mut parser = EventReader::new(bytes);
    while let Ok(xml_event) = &parser.next() {
        match xml_event {
            XmlEvent::StartElement { name: _name, attributes, namespace: _namespace } => {
                collector.append(&mut from_xml_start_element_attributes(&attributes, &parser)?)
            }
            XmlEvent::Comment(comment) => {
                collector.append(&mut find_urls(comment)
                    .iter()
                    .map(|&link| XmlLink {
                        url: link.to_string(),
                        location: parser.position(),
                        kind: XmlLinkType::Comment,
                    })
                    .collect()
                )
            }
            XmlEvent::Characters(chars) => {
                collector.append(&mut find_urls(chars)
                    .iter()
                    .map(|&link| XmlLink {
                        url: link.to_string(),
                        location: parser.position(),
                        kind: XmlLinkType::PlainText,
                    })
                    .collect()
                )
            }
            XmlEvent::EndDocument => {break}
            _ => {}
        }
    }
    
    Ok(collector)
}

fn from_xml_start_element_attributes(attributes: &Vec<OwnedAttribute>, parser: &EventReader<&[u8]>) -> Result<Vec<XmlLink>, XmlError> {
    let mut ret: Vec<XmlLink> = vec![];
    for attribute in attributes {
        let mut links = find_urls(&attribute.value)
            .iter().map(|&link| XmlLink {
            url: link.to_string(),
            location: parser.position(),
            kind: XmlLinkType::Attribute(attribute.clone()),
        }).collect();

        ret.append(&mut links);
    }
    Ok(ret)
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

    #[test]
    fn extract_all() {
        let links = extract_links(TEST_XLINK).unwrap();
        println!("{:?}", links);
        assert_eq!(3, links.len())
    }
}
