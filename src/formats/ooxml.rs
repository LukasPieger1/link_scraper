use std::io::{Cursor, Read};
use itertools::Itertools;
use thiserror::Error;
use xml::EventReader;
use xml::reader::XmlEvent;
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

pub struct OoxmlLocation {
    pub file: String,
    pub line: u64,
    pub pos: u64
}

/// Extracts all links from a given ooxml-file
///
/// Tries to filter out urls related to ooxml-functionalities, but might be a bit too aggressive at times
/// if there are links missing from the output, use [`extract_links_unfiltered`]
pub fn extract_links(bytes: &[u8]) -> Result<Vec<String>, OoxmlExtractionError> {
    let cur = Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cur)?;

    let mut links: Vec<String> = vec![];
    for file_name in archive.file_names().map(|name| name.to_owned()).collect_vec() {
        let mut file_content = vec![];
        archive.by_name(&file_name)?.read_to_end(&mut file_content)?;
        if file_content.is_empty() {
            continue;
        }

        if file_name.ends_with(".rels") {
            extract_links_from_rels_file(file_content.as_slice(), &mut links)?
        } else if file_name.ends_with(".xml") {
            extract_links_from_xml_file(file_content.as_slice(), &mut links)?
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
fn extract_links_from_rels_file(data: impl Read, collector: &mut Vec<String>) -> Result<(), OoxmlExtractionError> {
    let parser = EventReader::new(data);
    for e in parser {
        let event = e?;
        if let XmlEvent::StartElement { name: _, attributes, .. } = event {
            let attributes_with_potential_links = attributes.iter().filter(|att| &att.name.local_name != "Type");
            for attribute in attributes_with_potential_links {
                find_urls(&attribute.value).iter().for_each(|link| collector.push(link.to_string()))
            }
        }
    }
    Ok(())
}

/// Extracts links from given .xml file-text
///
/// All tags and tag-attributes are omitted to filter out functional urls.
/// This might be too aggressive in some cases though
fn extract_links_from_xml_file(data: impl Read, collector: &mut Vec<String>) -> Result<(), OoxmlExtractionError>{
    let parser = EventReader::new(data);
    for e in parser {
        let event = e?;
        let raw_text = match event {
            XmlEvent::Characters(str) => Some(str),
            XmlEvent::Whitespace(str) => Some(str),
            _ => None
        };
        if let Some(text) = raw_text {
            find_urls(&text).iter().for_each(|link| collector.push(link.to_string()));
        }
    }
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::include_bytes;
    use crate::link_extractor::{unique_and_sort};

    const TEST_DOCX: &[u8] = include_bytes!("../../test_files/docx/test.docx");
    const TEST_PPTX: &[u8] = include_bytes!("../../test_files/pptx/test.pptx");
    const TEST_XLSX: &[u8] = include_bytes!("../../test_files/xlsx/test.xlsx");

    #[test]
    pub fn docx_extraction_test() {
        let links = extract_links(TEST_DOCX).unwrap();
        assert_eq!(
            unique_and_sort(links.as_slice()),
            vec!["http://calibre-ebook.com/download", "http://embedded.link.de/", "http://iam.also.here", "http://test.comment.link", "https://music.youtube.com/watch?v=fsJ2QVjzwtQ&si=S4UQH23jwXIiZdad"]);
    }

    #[test]
    pub fn powerpoint_extraction_test() {
        let links = extract_links(TEST_PPTX).unwrap();
        assert_eq!(
            unique_and_sort(links.as_slice()),
            vec!["http://hyperlink.test.de/", "http://test.link.de/", "http://wurst.salat.de"]
        );
    }

    #[test]
    pub fn excel_extraction_test() {
        let links = extract_links(TEST_XLSX).unwrap();

        assert_eq!(
            unique_and_sort(links.as_slice()),
            vec!["http://xlsx.test.fail/", "https://antother.test/"]
        );
    }

    #[test]
    pub fn unfiltered_extraction_test() {
        let mut links = extract_links_unfiltered(TEST_DOCX).unwrap();
        links.sort();
        assert_eq!(links.len(), 130);
    }
}
