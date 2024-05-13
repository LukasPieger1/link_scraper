use std::fmt::{Display, Formatter};
use thiserror::Error;
use xml::attribute::OwnedAttribute;
use xml::common::TextPosition;
use crate::formats::xml::svg::SvgLinkType::{Attribute, Comment, NameSpace, Script, Text};
use crate::formats::xml::XmlLinkType;

#[derive(Error, Debug)]
pub enum SvgExtractionError {
    #[error(transparent)]
    XmlExtractionError(#[from] crate::formats::xml::XmlExtractionError),
}

#[derive(Debug, Clone)]
pub struct SvgLink {
    pub url: String,
    pub location: TextPosition,
    pub kind: SvgLinkType,
}

#[derive(Debug, Clone)]
pub enum SvgLinkType {
    Attribute(OwnedAttribute),
    Comment,
    Text,
    Script,
    NameSpace(String),
}

impl Display for SvgLink {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.url)
    }
}

#[derive(Debug, Clone)]
pub struct SvgLinkLocation {
    pub file: String,
    pub position: TextPosition,
}

pub fn extract_links(bytes: &[u8]) -> Result<Vec<SvgLink>, SvgExtractionError> {
    Ok(crate::formats::xml::extract_links(bytes)?
        .into_iter()
        .map(|link| SvgLink {
            url: link.url,
            location: link.location,
            kind: match link.kind {
                XmlLinkType::Attribute(attribute) => {Attribute(attribute)}
                XmlLinkType::Comment => {Comment}
                XmlLinkType::PlainText(_) => {Text}
                XmlLinkType::CData(_) => {Script}
                XmlLinkType::NameSpace(ns) => {NameSpace(ns)}
            },
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SVG: &[u8] = include_bytes!("../../../test_files/svg/test.svg");
    #[test]
    fn extract_links_from_svg() {
        let links = extract_links(TEST_SVG).unwrap();
        println!("{:?}", links);
        assert_eq!(links.len(), 10)
    }
}
