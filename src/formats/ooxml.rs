use std::fmt::{Display, Formatter};
use std::io::{Cursor, Read};
use itertools::Itertools;
use thiserror::Error;
use xml::common::{Position, TextPosition};
use xml::EventReader;
use xml::reader::XmlEvent;
use crate::formats::ooxml::OoxmlLinkKind::{Comment, Hyperlink};
use crate::link_extractor::find_urls;

#[derive(Error, Debug)]
pub enum OoxmlExtractionError {
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

#[derive(Debug, Clone)]
pub struct OoxmlLinkLocation {
    pub file: String,
    pub position: TextPosition
}

#[derive(Debug, Clone, Copy)]
pub enum OoxmlLinkKind {
    PlainText,
    Hyperlink,
    Comment
}

/// Extracts all links from a given ooxml-file
///
/// Tries to filter out urls related to ooxml-functionalities, but might be a bit too aggressive at times
/// if there are links missing from the output, use [`extract_links_unfiltered`]
pub fn extract_links(bytes: &[u8]) -> Result<Vec<OoxmlLink>, OoxmlExtractionError> {
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
            extract_links_from_rels_file(file_content.as_slice(), &file_name, &mut links)?
        } else if file_name.ends_with(".xml") {
            extract_links_from_xml_file(file_content.as_slice(), &file_name, &mut links)?
        }
    }
    
    Ok(links)
}

/// Extracts all links from a given ooxml file.
///
/// To avoid getting urls related to ooxml-functionalities use [`extract_links`] instead.
pub fn extract_links_unfiltered(bytes: &[u8]) -> Result<Vec<String>, OoxmlExtractionError> {
    crate::formats::compressed_formats_common::extract_links_unfiltered(bytes)
        .map_err(|e| OoxmlExtractionError::from(e))
}

/// Extracts links from given .rels file
fn extract_links_from_rels_file(data: impl Read, file_name: &str, collector: &mut Vec<OoxmlLink>) -> Result<(), OoxmlExtractionError> {
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

/// Extracts links from given .xml file-text
///
/// All tags and tag-attributes are omitted to filter out functional urls.
/// This might be too aggressive in some cases though
fn extract_links_from_xml_file(data: impl Read, file_name: &str, collector: &mut Vec<OoxmlLink>) -> Result<(), OoxmlExtractionError>{
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

    const TEST_DOCX: &[u8] = include_bytes!("../../test_files/docx/test.docx");
    const TEST_PPTX: &[u8] = include_bytes!("../../test_files/pptx/test.pptx");
    const TEST_XLSX: &[u8] = include_bytes!("../../test_files/xlsx/test.xlsx");

    #[test]
    pub fn docx_extraction_test() {
        let links = extract_links(TEST_DOCX).unwrap();
        println!("{:?}", links);
        assert_eq!(links.len(), 5);
    }

    #[test]
    pub fn powerpoint_extraction_test() {
        let links = extract_links(TEST_PPTX).unwrap();
        assert_eq!(links.len(), 3);
    }

    #[test]
    pub fn excel_extraction_test() {
        let links = extract_links(TEST_XLSX).unwrap();
        assert_eq!(links.len(), 3);
    }

    #[test]
    pub fn unfiltered_extraction_test() {
        let mut links = extract_links_unfiltered(TEST_DOCX).unwrap();
        links.sort();
        assert_eq!(links.len(), 130);
    }
}
