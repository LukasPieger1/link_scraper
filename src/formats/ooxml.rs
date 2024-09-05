use std::fmt::{Display, Formatter};
use std::io::{Cursor};
use itertools::Itertools;
use thiserror::Error;
use xml::common::{Position, TextPosition};
use xml::EventReader;
use xml::reader::XmlEvent;
use crate::formats::ooxml::OoxmlLinkKind::{Comment, Hyperlink};
use crate::gen_scrape_from_file;
use crate::helpers::find_urls;

/// Scrapes all links from a given ooxml-file
///
/// Tries to filter out urls related to ooxml-functionalities, but might be a bit too aggressive at times
/// if there are links missing from the output, use [`scrape_unfiltered`]
pub fn scrape(bytes: &[u8]) -> Result<Vec<OoxmlLink>, OoxmlScrapingError> {
    let cur = Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cur)?;

    let mut links: Vec<OoxmlLink> = vec![];
    for file_name in archive.file_names().map(|name| name.to_owned()).collect_vec() {
        let mut file_content = vec![];
        archive.by_name(&file_name)?.read_to_end(&mut file_content)?;
        if file_content.is_empty() {
            continue;
        }

        if file_name.ends_with(".rels") {
            scrape_from_rels_file(file_content.as_slice(), &file_name, &mut links)?
        } else if file_name.ends_with(".xml") {
            scrape_from_xml_file(file_content.as_slice(), &file_name, &mut links)?
        }
    }

    Ok(links)
}
gen_scrape_from_file!(Result<Vec<OoxmlLink>, OoxmlScrapingError>);

#[derive(Error, Debug)]
pub enum OoxmlScrapingError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    XmlReaderError(#[from] xml::reader::Error),
    #[error(transparent)]
    ZipError(#[from] zip::result::ZipError),
}

#[derive(Debug, Clone)]
pub struct OoxmlLink {
    pub url: String,
    pub location: OoxmlLinkLocation,
    pub kind: OoxmlLinkKind,
}

impl Display for OoxmlLink {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.url)
    }
}

/// This Location references the location in the unzipped ooxml file-structure.
#[derive(Debug, Clone)]
pub struct OoxmlLinkLocation {
    pub file: String,
    pub position: TextPosition
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OoxmlLinkKind {
    /// The link is contained as Text inside the document
    PlainText,
    /// The link is contained as a Hyperlink inside the document
    Hyperlink,
    /// The link is contained as a Comment added to the document
    Comment
}

/// Scrapes all links from a given ooxml file.
///
/// To avoid getting urls related to ooxml-functionalities use [`scrape`] instead.
pub fn scrape_unfiltered(bytes: &[u8]) -> Result<Vec<String>, OoxmlScrapingError> {
    crate::formats::compressed_formats_common::scrape_unfiltered(bytes)
        .map_err(|e| OoxmlScrapingError::from(e))
}

/// Scrapes links from given .rels file
fn scrape_from_rels_file(data: impl Read, file_name: &str, collector: &mut Vec<OoxmlLink>) -> Result<(), OoxmlScrapingError> {
    let mut parser = EventReader::new(data);
    while let Ok(xml_event) = &parser.next() {
        if let XmlEvent::StartElement { name: _, attributes, .. } = xml_event {
            let attributes_with_potential_links = attributes.iter().filter(|att| &att.name.local_name != "Type");
            for attribute in attributes_with_potential_links {
                find_urls(&attribute.value).iter().for_each(|link| collector.push(OoxmlLink {
                    url: link.as_str().to_string(),
                    location: OoxmlLinkLocation { file: file_name.to_string(), position: parser.position()},
                    kind: Hyperlink
                }))
            }
        }

        if let XmlEvent::EndDocument{} = xml_event { break }
    }
    Ok(())
}

/// Scrapes links from given .xml file-text
///
/// All tags and tag-attributes are omitted to filter out functional urls.
/// This might be too aggressive in some cases though
fn scrape_from_xml_file(data: impl Read, file_name: &str, collector: &mut Vec<OoxmlLink>) -> Result<(), OoxmlScrapingError>{
    let mut parser = EventReader::new(data);
    while let Ok(xml_event) = &parser.next() {
        let raw_text = match xml_event {
            XmlEvent::Characters(str) => Some(str),
            XmlEvent::Whitespace(str) => Some(str),
            _ => None
        };
        if let Some(text) = raw_text {
            find_urls(&text).iter().for_each(|link| collector.push(OoxmlLink {
                url: link.as_str().to_string(),
                location: OoxmlLinkLocation { file: file_name.to_string(), position: parser.position()},
                kind: if file_name == "word/comments.xml" { Comment } else { Hyperlink }
            }));
        }

        if let XmlEvent::EndDocument{} = xml_event { break }
    }
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::include_bytes;

    const TEST_DOCX: &[u8] = include_bytes!("../../test_files/ooxml/docx_test.docx");
    const TEST_PPTX: &[u8] = include_bytes!("../../test_files/ooxml/pptx_test.pptx");
    const TEST_XLSX: &[u8] = include_bytes!("../../test_files/ooxml/xlsx_test.xlsx");

    #[test]
    pub fn scrape_docx_test() {
        let links = scrape(TEST_DOCX).unwrap();
        println!("{:?}", links);
        assert!(links.iter().any(|it| it.url == "https://hyperlink.test.com/" && it.kind == Hyperlink));
        assert!(links.iter().any(|it| it.url == "https://comment.test.com" && it.kind == Comment));
    }

    #[test]
    pub fn scrape_pptx_test() {
        let links = scrape(TEST_PPTX).unwrap();
        println!("{:?}", links);
        assert!(links.iter().any(|it| it.url == "https://hyperlink.test.com/" && it.kind == Hyperlink));
    }

    #[test]
    pub fn scrape_xlsx_test() {
        let links = scrape(TEST_XLSX).unwrap();
        println!("{:?}", links);
        assert!(links.iter().any(|it| it.url == "https://hyperlink.test.com/" && it.kind == Hyperlink));
    }

    #[test]
    pub fn scrape_unfiltered_test() {
        let mut links = scrape_unfiltered(TEST_DOCX).unwrap();
        links.sort();
        assert_eq!(links.len(), 50);
    }
}
