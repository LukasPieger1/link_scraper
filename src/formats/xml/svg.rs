use std::fmt::{Display, Formatter};
use thiserror::Error;
use xml::attribute::OwnedAttribute;
use xml::common::TextPosition;
use crate::formats::xml::svg::SvgLinkType::{Attribute, Comment, NameSpace, Script, Text};
use crate::formats::xml::XmlLinkType;
use crate::gen_scrape_from_file;

pub fn scrape(bytes: &[u8]) -> Result<Vec<SvgLink>, SvgScrapingError> {
    Ok(crate::formats::xml::scrape(bytes)?
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
gen_scrape_from_file!(Result<Vec<SvgLink>, SvgScrapingError>);

#[derive(Error, Debug)]
pub enum SvgScrapingError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    XmlScrapingError(#[from] crate::formats::xml::XmlScrapingError),
}

#[derive(Debug, Clone)]
pub struct SvgLink {
    pub url: String,
    pub location: TextPosition,
    pub kind: SvgLinkType,
}

#[derive(Debug, Clone)]
pub enum SvgLinkType {
    /// The link is inside a xml-attribute <br/>
    /// Example: `<a href="https://link.example.com">`
    Attribute(OwnedAttribute),
    /// The link is inside a xml-comment <br/>
    /// Example: `<!--Just a comment with a link to https://link.example.com-->`
    Comment,
    /// The link is inside a plaintext portion<br/>
    /// Example: `<p> Just a comment with a link to https://link.example.com </p>`
    Text,
    /// The link is inside a script portion<br/>
    /// Example:
    /// ```<t>
    /// <script type="text/ecmascript">
    ///     <![CDATA[
    ///         var scriptLink = "https://link.example.com";
    ///     ]]>
    /// </script>```
    Script,
    /// This link is a reference to a xml-namespace<br/>
    /// Example: `<root xmlns="https://link.example.com">`
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

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SVG: &[u8] = include_bytes!("../../../test_files/svg/test.svg");
    #[test]
    fn scrape_svg_test() {
        let links = scrape(TEST_SVG).unwrap();
        println!("{:?}", links);
        assert_eq!(links.len(), 10)
    }
}
