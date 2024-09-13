use crate::formats::xml::svg::SvgLinkKind::{Attribute, Comment, NameSpace, Script, Text};
use crate::formats::xml::XmlLinkKind;
use crate::gen_scrape_froms;
use std::fmt::{Display, Formatter};
use std::io::Read;
use thiserror::Error;
use xml::attribute::OwnedAttribute;
use xml::common::TextPosition;

pub fn scrape<R>(reader: R) -> Result<Vec<SvgLink>, SvgScrapingError>
where
    R: Read,
{
    Ok(crate::formats::xml::scrape(reader)?
        .into_iter()
        .map(|link| SvgLink {
            url: link.url,
            location: link.location,
            kind: match link.kind {
                XmlLinkKind::Attribute(attribute) => Attribute(attribute),
                XmlLinkKind::Comment => Comment,
                XmlLinkKind::PlainText(_) => Text,
                XmlLinkKind::CData(_) => Script,
                XmlLinkKind::NameSpace(ns) => NameSpace(ns),
            },
        })
        .collect())
}
gen_scrape_froms!(scrape(Read) -> Result<Vec<SvgLink>, SvgScrapingError>);

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
    pub kind: SvgLinkKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SvgLinkKind {
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
    /// ```text
    /// <script type="text/ecmascript">
    ///     <![CDATA[
    ///         var scriptLink = "https://link.example.com";
    ///     ]]>
    /// </script>
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

    const TEST_SVG: &[u8] = include_bytes!("../../../test_files/xml/svg_test.svg");
    #[test]
    fn scrape_svg_test() {
        let links = scrape(TEST_SVG).unwrap();
        println!("{:?}", links);
        assert!(links
            .iter()
            .any(|it| it.url == "https://cdata.test.com/insideACodeSnippet"
                && matches!(it.kind, Script)));
        assert!(links
            .iter()
            .any(|it| it.url == "http://www.w3.org/2000/svg" && matches!(it.kind, NameSpace(_))));
    }
}
