pub mod elements;

use itertools::Itertools;
use thiserror::Error;
use xml::attribute::OwnedAttribute;
use xml::common::{Position, TextPosition};
use xml::EventReader;
use xml::reader::XmlEvent;
use crate::formats::xml::xlink::elements::{XlinkElement, XlinkExtendedElement, XlinkSimpleElement};
use crate::formats::xml::xlink::XLinkFormatError::{ArcOutsideOfExtendedError, ExtendedInsideOfExtendedError, LocatorOutsideOfExtendedError, ResourceOutsideOfExtendedError, SimpleInsideOfExtendedError};
use crate::formats::xml::XmlStartElement;
use crate::gen_scrape_from_file;
use crate::link_scraper::find_urls;

#[derive(Error, Debug)]
pub enum XLinkFormatError {
    #[error("Unknown xlink:type value.")]
    UnknownTypeError(String),
    #[error("Xlink-element is missing a required attribute.")]
    MissingRequiredAttributeError(String),
    #[error("Found a locator-element outside of an extended element.")]
    LocatorOutsideOfExtendedError,
    #[error("Found an arc-element outside of an extended element.")]
    ArcOutsideOfExtendedError,
    #[error("Found a resource-element outside of an extended element.")]
    ResourceOutsideOfExtendedError,
    #[error("Found a simple-element inside of an extended element.")]
    SimpleInsideOfExtendedError,
    #[error("Found a extended-element inside of an extended element.")]
    ExtendedInsideOfExtendedError,
    #[error(transparent)]
    XmlReaderError(#[from] xml::reader::Error)
}

fn get_xlink_attribute_value(key: &str, attributes: &Vec<OwnedAttribute>) -> Option<String> {
    attributes.iter()
        .find( |attribute|
            attribute.name.local_name == key && attribute.name.namespace == Some(XLINK_NAMESPACE.to_string()))
        .map(|href_attribute| href_attribute.value.to_string())
}

#[derive(Debug)]
pub struct XLinkLink {
    pub url: String,
    pub location: TextPosition,
    pub kind: XLinkLinkType
}

#[derive(Debug, Copy, Clone)]
pub enum XLinkLinkType {
    Simple,
    External,
    Role,
    ArcRole,
}

static XLINK_NAMESPACE: &str = "http://www.w3.org/1999/xlink";

pub fn scrape(bytes: &[u8]) -> Result<Vec<XLinkLink>, XLinkFormatError> {
    let mut collector: Vec<XLinkLink> = vec![];
    
    let mut parser = EventReader::new(bytes);
    while let Ok(xml_event) = &parser.next() {
        match xml_event {
            XmlEvent::StartElement { name, attributes, namespace } => { 
                let mut list = scrape_from_start_element(XmlStartElement { name, attributes, _namespace: namespace }, &mut parser)?;
                collector.append(&mut list)
            }
            XmlEvent::EndDocument => break,
            _ => {}
        }
    }

    Ok(collector)
}
gen_scrape_from_file!(Result<Vec<XLinkLink>, XLinkFormatError>);

fn scrape_from_start_element(xml_start_element: XmlStartElement, mut parser: &mut EventReader<&[u8]>) -> Result<Vec<XLinkLink>, XLinkFormatError> {
    let Some(xlink_element) = XlinkElement::try_from_xml_start_element(xml_start_element)?
    else { return Ok(vec![]) };

    match xlink_element {
        XlinkElement::Simple(element) => Ok(scrape_from_xlink_simple(element, &parser)),
        XlinkElement::Extended(element) => scrape_from_xlink_extended(element, &mut parser),
        XlinkElement::Locator(_) => Err(LocatorOutsideOfExtendedError),
        XlinkElement::Arc(_) => Err(ArcOutsideOfExtendedError),
        XlinkElement::Resource(_) => Err(ResourceOutsideOfExtendedError),
        XlinkElement::Title(_) => Ok(vec![])
    }
}

fn scrape_from_option_string(role: Option<String>, link_type: XLinkLinkType, position: TextPosition) -> Vec<XLinkLink> {
    let Some(role) = role
        else { return vec![] };
    let links = find_urls(&role).iter()
        .map(|link| XLinkLink {
            url: link.as_str().to_string(),
            location: position,
            kind: link_type,
        }).collect_vec();
    links
}

fn scrape_from_xlink_extended(xlink_extended_element: XlinkExtendedElement, parser: &mut EventReader<&[u8]>) -> Result<Vec<XLinkLink>, XLinkFormatError> {
    let mut ret: Vec<XLinkLink> = scrape_from_option_string(xlink_extended_element.role, XLinkLinkType::Role, parser.position());

    while let Ok(xml_event) = &parser.next() {
        let mut links = match xml_event {
            XmlEvent::StartElement { name, attributes, namespace } => {
                let Some(xlink_element) = 
                    XlinkElement::try_from_xml_start_element(XmlStartElement{ name, attributes, _namespace: namespace })?
                else { continue };
                
                match xlink_element {
                    XlinkElement::Simple(_) => Err(SimpleInsideOfExtendedError),
                    XlinkElement::Extended(_) => Err(ExtendedInsideOfExtendedError),
                    XlinkElement::Locator(element) => {
                        let mut locator_links = vec![];
                        
                        locator_links.push(XLinkLink {
                            url: element.href,
                            location: parser.position(),
                            kind: XLinkLinkType::External
                        });
                        locator_links.append(&mut scrape_from_option_string(element.role, XLinkLinkType::Role, parser.position()));
                        
                        Ok(locator_links)
                    }
                    XlinkElement::Arc(element) => {
                        Ok(scrape_from_option_string(element.arcrole, XLinkLinkType::ArcRole, parser.position()))
                    },
                    XlinkElement::Resource(element) => {
                        Ok(scrape_from_option_string(element.role, XLinkLinkType::Role, parser.position()))
                    },
                    XlinkElement::Title(_) => Ok(vec![])
                }?
            }
            XmlEvent::EndElement { name } => {
                if name.eq(xlink_extended_element.xml.name) {
                    break
                } else { vec![] }
            }
            _ => vec![]
        };
        ret.append(&mut links);
    }

    Ok(ret)
}

fn scrape_from_xlink_simple(xlink_element: XlinkSimpleElement, parser: &EventReader<&[u8]>) -> Vec<XLinkLink> {
    let mut ret = scrape_from_option_string(xlink_element.href, XLinkLinkType::Simple, parser.position());
    ret.append(&mut scrape_from_option_string(xlink_element.arcrole, XLinkLinkType::ArcRole, parser.position()));
    ret.append(&mut scrape_from_option_string(xlink_element.role, XLinkLinkType::Role, parser.position()));
    ret
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_XLINK: &[u8] = include_bytes!("../../../../test_files/xml/xlink_test.xml");

    #[test]
    fn scrape_xlink_test() {
        let links = scrape(TEST_XLINK).unwrap();
        println!("{:?}", links);
        assert_eq!(2, links.len())
    }
}
