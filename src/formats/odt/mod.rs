use std::io::{Cursor, Read};
use itertools::Itertools;
use thiserror::Error;
use xml::EventReader;
use xml::reader::XmlEvent;
use crate::formats::ooxml::DocxExtractionError;
use crate::link_extractor::find_links;

#[derive(Error, Debug)]
pub enum OdtExtractionError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    XmlReaderError(#[from] xml::reader::Error),
    #[error(transparent)]
    ZipError(#[from] zip::result::ZipError),
}

/// Extracts all links from a given ooxml-file
///
/// Tries to filter out urls related to ooxml-functionalities, but might be a bit too aggressive at times
/// if there are links missing from the output, use [`extract_links_unfiltered`]
pub fn extract_links(bytes: &[u8]) -> Result<Vec<String>, OdtExtractionError> {
    let cur = Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cur)?;

    let mut links: Vec<String> = vec![];
    for file_name in archive.file_names().map(|name| name.to_owned()).collect_vec() {
        let mut file_content = vec![];
        archive.by_name(&file_name)?.read_to_end(&mut file_content)?;

        if file_name.ends_with(".xml") {
            extract_links_from_xml_file(file_content.as_slice(), &mut links)?
        }
    }

    Ok(links)
}

/// Extracts all links from a given odt file.
///
/// To avoid getting urls related to odt-functionalities use [`crate::formats::ooxml::extract_links`] instead.
pub fn extract_links_unfiltered(bytes: &[u8]) -> Result<Vec<String>, DocxExtractionError> {
    crate::formats::compressed_formats_common::extract_links_unfiltered(bytes)
}

/// Extracts links from given .xml file-text
///
/// All tags and tag-attributes are omitted to filter out functional urls.
/// This might be too aggressive in some cases though
fn extract_links_from_xml_file(data: impl Read, collector: &mut Vec<String>) -> Result<(), OdtExtractionError> {
    let parser = EventReader::new(data);
    for e in parser {
        let event = e?;
        let raw_text = match event {
            XmlEvent::StartElement {name, attributes, ..} => 
                if name.local_name == "a" {
                    attributes.iter()
                        .find(|attr| attr.name.local_name == "href")
                        .map(|href| href.value.to_string())
                } else {
                    None
                }
            XmlEvent::Characters(str) => Some(str),
            XmlEvent::Whitespace(str) => Some(str),
            _ => None
        };
        if let Some(text) = raw_text {
            find_links(&text).iter().for_each(|link| collector.push(link.to_string()));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::include_bytes;
    use crate::link_extractor::{unique_and_sort};

    const TEST_ODT: &[u8] = include_bytes!("../../../assets/examples/odt/file-sample_1MB.odt");

    #[test]
    pub fn docx_extraction_test() {
        let links = extract_links(TEST_ODT).unwrap();
        assert_eq!(
            unique_and_sort(links.as_slice()),
            vec!["http://comment.link.test", "https://hyperlink.in.comment.test/", "https://hyperlink.test.de/", "https://plaintext.link.test/", "https://products.office.com/en-us/word"]);
    }

    #[test]
    pub fn unfiltered_extraction_test() {
        let links = extract_links_unfiltered(TEST_ODT).unwrap();
        let links = unique_and_sort(&links);
        assert_eq!(links.len(), 28);
    }
}
