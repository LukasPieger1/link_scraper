use std::io::{BufReader, Read};
use thiserror::Error;
use crate::link_extractor::find_urls;

#[derive(Error, Debug)]
pub enum TextFileExtractionError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

pub struct TxtLocation {
    pub line: u64,
    pub pos: u64
}

pub fn extract_links(bytes: &[u8]) -> Result<Vec<String>, TextFileExtractionError> {
    let mut buf_reader = BufReader::new(bytes);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;
    Ok(find_urls(&contents).iter().map(|link| link.to_string()).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_XML: &[u8] = include_bytes!("../../test_files/xml/test.xml");
    #[test]
    fn get_some_website() {
        let links = extract_links(TEST_XML).unwrap();
        assert_eq!(links, vec!["http://www.placeholder-name-here.com/schema/"])
    }
}
