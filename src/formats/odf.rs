use std::io::{Cursor, Read};
use itertools::Itertools;
use thiserror::Error;
use xml::common::{Position, TextPosition};
use xml::EventReader;
use xml::reader::XmlEvent;
use crate::formats::odf::OdfLinkKind::{Hyperlink, PlainText};
use crate::link_extractor::find_urls;

#[derive(Error, Debug)]
pub enum OdfExtractionError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    XmlReaderError(#[from] xml::reader::Error),
    #[error(transparent)]
    ZipError(#[from] zip::result::ZipError),
}

#[derive(Debug, Clone)]
pub struct OdfLink {
    pub url: String,
    pub location: OdfLinkLocation,
    pub kind: OdfLinkKind,
}

#[derive(Debug, Clone)]
pub struct OdfLinkLocation {
    pub file: String,
    pub position: TextPosition
}

#[derive(Debug, Clone, Copy)]
pub enum OdfLinkKind {
    PlainText,
    Hyperlink
}

/// Extracts all links from a given ooxml-file
///
/// Tries to filter out urls related to ooxml-functionalities, but might be a bit too aggressive at times
/// if there are links missing from the output, use [`extract_links_unfiltered`]
pub fn extract_links(bytes: &[u8]) -> Result<Vec<OdfLink>, OdfExtractionError> {
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
            extract_links_from_xml_file(file_content.as_slice(), &file_name, &mut links)?
        }
    }

    Ok(links)
}

/// Extracts all links from a given odf file.
///
/// To avoid getting urls related to odf-functionalities use [`crate::formats::ooxml::extract_links`] instead.
pub fn extract_links_unfiltered(bytes: &[u8]) -> Result<Vec<String>, OdfExtractionError> {
    crate::formats::compressed_formats_common::extract_links_unfiltered(bytes)
        .map_err(|e| OdfExtractionError::from(e))
}

/// Extracts links from given .xml file-text
///
/// All tags and tag-attributes are omitted to filter out functional urls.
/// This might be too aggressive in some cases though
fn extract_links_from_xml_file(data: impl Read, filename: &str, collector: &mut Vec<OdfLink>) -> Result<(), OdfExtractionError> {
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
                    .map(|&link| OdfLink {
                        url: link.to_string(),
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
    use crate::link_extractor::{unique_and_sort};

    const TEST_ODT: &[u8] = include_bytes!("../../test_files/odt/test.odt");

    #[test]
    pub fn docx_extraction_test() {
        let links = extract_links(TEST_ODT).unwrap();
        println!("{:?}", links)
    }

    #[test]
    pub fn unfiltered_extraction_test() {
        let links = extract_links_unfiltered(TEST_ODT).unwrap();
        let links = unique_and_sort(&links);
        assert_eq!(links.len(), 28);
    }
}
