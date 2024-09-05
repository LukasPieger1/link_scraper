use std::fmt::{Display, Formatter};
use std::io::{Cursor};
use itertools::Itertools;
use thiserror::Error;
use xml::common::{Position, TextPosition};
use xml::EventReader;
use xml::reader::XmlEvent;
use crate::formats::odf::OdfLinkKind::{Hyperlink, PlainText};
use crate::gen_scrape_from_file;
use crate::helpers::find_urls;

/// Scrapes all links from a given ooxml-file
///
/// Tries to filter out urls related to ooxml-functionalities, but might be a bit too aggressive at times
/// if there are links missing from the output, use [`scrape_unfiltered`]
pub fn scrape(bytes: &[u8]) -> Result<Vec<OdfLink>, OdfScrapingError> {
    let cur = Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cur)?;

    let mut links: Vec<OdfLink> = vec![];
    for file_name in archive.file_names().map(|name| name.to_owned()).collect_vec() {
        let mut file_content = vec![];
        archive.by_name(&file_name)?.read_to_end(&mut file_content)?;
        if file_content.is_empty() {
            continue;
        }

        if file_name.ends_with(".xml") {
            scrape_from_xml_file(file_content.as_slice(), &file_name, &mut links)?
        }
    }

    Ok(links)
}
gen_scrape_from_file!(Result<Vec<OdfLink>, OdfScrapingError>);

#[derive(Error, Debug)]
pub enum OdfScrapingError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    XmlReaderError(#[from] xml::reader::Error),
    #[error(transparent)]
    ZipError(#[from] zip::result::ZipError),
}

#[derive(Debug, Clone, PartialEq)]
pub struct OdfLink {
    pub url: String,
    pub location: OdfLinkLocation,
    pub kind: OdfLinkKind,
}

impl Display for OdfLink {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.url)
    }
}

/// This Location references the location in the unzipped odf file-structure.
#[derive(Debug, Clone, PartialEq)]
pub struct OdfLinkLocation {
    pub file: String,
    pub position: TextPosition
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OdfLinkKind {
    /// The link is contained as Text or as a Comment inside the document
    PlainText,
    /// The link is contained as a Hyperlink inside the document
    Hyperlink
}

/// Scrapes all links from a given odf file.
///
/// To avoid getting urls related to odf-functionalities use [`crate::formats::ooxml::scrape`] instead.
pub fn scrape_unfiltered(bytes: &[u8]) -> Result<Vec<String>, OdfScrapingError> {
    crate::formats::compressed_formats_common::scrape_unfiltered(bytes)
        .map_err(|e| OdfScrapingError::from(e))
}

/// Scrapes links from given .xml file-text
///
/// All tags and tag-attributes are omitted to filter out functional urls.
/// This might be too aggressive in some cases though
fn scrape_from_xml_file(data: impl Read, filename: &str, collector: &mut Vec<OdfLink>) -> Result<(), OdfScrapingError> {
    let mut parser = EventReader::new(data);

    while let Ok(xml_event) = &parser.next() {
        match xml_event {
            XmlEvent::StartElement { name, attributes, .. } => {
                if name.local_name != "a" { continue }

                let maybe_href = &attributes.iter()
                    .find(|&attr| attr.name.local_name == "href");
                if let Some(href) = maybe_href {
                    let link = OdfLink {
                        url: href.value.to_string(),
                        location: OdfLinkLocation { file: filename.to_string(), position: parser.position() },
                        kind: Hyperlink,
                    };
                    collector.push(link);
                }
            },
            XmlEvent::Characters(chars) => {
                collector.append(&mut find_urls(&chars)
                    .iter()
                    .map(|link| OdfLink {
                        url: link.as_str().to_string(),
                        location: OdfLinkLocation {file: filename.to_string(), position: parser.position()},
                        kind: PlainText,
                    })
                    .collect()
                )
            }
            XmlEvent::EndDocument => { break }
            _ => {}
        };
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::include_bytes;

    const TEST_ODT: &[u8] = include_bytes!("../../test_files/odf/odt_test.odt");
    const TEST_ODS: &[u8] = include_bytes!("../../test_files/odf/ods_test.ods");
    const TEST_ODP: &[u8] = include_bytes!("../../test_files/odf/odp_test.odp");
    const TEST_OTT: &[u8] = include_bytes!("../../test_files/odf/ott_test.ott");

    #[test]
    pub fn scrape_odt_test() {
        let links = scrape(TEST_ODT).unwrap();
        println!("{:?}", links);
        assert!(links.iter().any(|it| it.url == "https://plaintext.test.com" && it.kind == PlainText));
        assert!(links.iter().any(|it| it.url == "https://hyperlink.test.com/" && it.kind == Hyperlink));
    }

    #[test]
    pub fn scrape_ods_test() {
        let links = scrape(TEST_ODS).unwrap();
        println!("{:?}", links);
        assert!(links.iter().any(|it| it.url == "https://plaintext.test.com" && it.kind == PlainText));
        assert!(links.iter().any(|it| it.url == "https://hyperlink.test.com/" && it.kind == Hyperlink));
    }

    #[test]
    pub fn scrape_odp_test() {
        let links = scrape(TEST_ODP).unwrap();
        println!("{:?}", links);
        assert!(links.iter().any(|it| it.url == "https://plaintext.test.com" && it.kind == PlainText));
        assert!(links.iter().any(|it| it.url == "https://hyperlink.test.com/" && it.kind == Hyperlink));
    }

    #[test]
    pub fn scrape_ott_test() {
        let links = scrape(TEST_OTT).unwrap();
        println!("{:?}", links);
        assert!(links.iter().any(|it| it.url == "https://plaintext.test.com" && it.kind == PlainText));
        assert!(links.iter().any(|it| it.url == "https://hyperlink.test.com/" && it.kind == Hyperlink));
    }

    #[test]
    pub fn scrape_unfiltered_test() {
        let links = scrape_unfiltered(TEST_ODT).unwrap();
        assert_eq!(links.len(), 47);
    }
}
