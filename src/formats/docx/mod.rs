use std::io::{Cursor, Read};
use itertools::Itertools;
use thiserror::Error;
use xml::EventReader;
use xml::reader::XmlEvent;
use crate::link_extractor::find_links;

#[derive(Error, Debug)]
pub enum DocxExtractionError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    XmlReaderError(#[from] xml::reader::Error),
    #[error(transparent)]
    ZipError(#[from] zip::result::ZipError),
}

/// Extracts all links from a given docx-file
///
/// Tries to filter out urls related to docx-functionalities, but might be a bit too aggressive at times
/// if there are links missing from the output, use [`extract_links_unfiltered`]
pub fn extract_links(bytes: &[u8]) -> Result<Vec<String>, DocxExtractionError> {
    let cur = Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cur)?;

    let mut links: Vec<String> = vec![];
    for file_name in archive.file_names().map(|name| name.to_owned()).collect_vec() {
        let mut file_content = vec![];
        archive.by_name(&file_name)?.read_to_end(&mut file_content)?;

        if file_name.ends_with(".rels") {
            extract_links_from_rels_file(file_content.as_slice(), &mut links)?
        } else if file_name.ends_with(".xml") {
            extract_links_from_xml_file(file_content.as_slice(), &mut links)?
        }
    }

    Ok(links)
}

/// Extracts all links from a given docx file.
///
/// To avoid getting urls related to the docx-functionalities use [`extract_links`] instead.
pub fn extract_links_unfiltered(bytes: &[u8]) -> Result<Vec<String>, DocxExtractionError> {
    let cur = Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cur)?;

    let mut links: Vec<String> = vec![];
    for file_name in archive.file_names().map(|name| name.to_owned()).collect_vec() {
        let mut file_content = String::new();
        if archive.by_name(&file_name)?.read_to_string(&mut file_content).is_err() {continue}

        find_links(&file_content).iter().for_each(|link| links.push(link.to_string()))
    }

    Ok(links)
}

/// Extracts links from given .rels file
fn extract_links_from_rels_file(data: impl Read, collector: &mut Vec<String>) -> Result<(), DocxExtractionError> {
    let parser = EventReader::new(data);
    for e in parser {
        let event = e?;
        if let XmlEvent::StartElement { name: _, attributes, .. } = event {
            let attributes_with_potential_links = attributes.iter().filter(|att| &att.name.local_name != "Type");
            for attribute in attributes_with_potential_links {
                find_links(&attribute.value).iter().for_each(|link| collector.push(link.to_string()))
            }
        }
    }
    Ok(())
}

/// Extracts links from given .xml file-text
///
/// All tags and tag-attributes are omitted to filter out functional urls.
/// This might be too aggressive in some cases though
fn extract_links_from_xml_file(data: impl Read, collector: &mut Vec<String>) -> Result<(), DocxExtractionError>{
    let parser = EventReader::new(data);
    let mut raw_text = String::new();
    for e in parser {
        let event = e?;
        match event {
            XmlEvent::Characters(str) => {raw_text += &str}
            XmlEvent::Whitespace(str) => {raw_text += &str}
            _ => {}
        }
    }
    find_links(&raw_text).iter().for_each(|link| collector.push(link.to_string()));
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::include_bytes;

    const TEST_DOCX: &[u8] = include_bytes!("../../../assets/examples/docx/demo.docx");

    #[test]
    pub fn extraction_test() {
        let mut links = extract_links(TEST_DOCX).unwrap();
        links.sort();
        assert_eq!(links, vec!["http://calibre-ebook.com/download", "http://embedded.link.de/", "http://iam.also.here", "http://test.comment.link", "https://music.youtube.com/watch?v=fsJ2QVjzwtQ&si=S4UQH23jwXIiZdad"]);
    }

    #[test]
    pub fn unfiltered_extraction_test() {
        let mut links = extract_links_unfiltered(TEST_DOCX).unwrap();
        links.sort();
        assert_eq!(links.len(), 130);
    }
}
