use std::io::{BufReader, Read};
use thiserror::Error;
use crate::link_extractor::find_links;

#[derive(Error, Debug)]
pub enum XmlExtractionError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

pub fn extract_links(bytes: &[u8]) -> Result<Vec<String>, XmlExtractionError> {
    let mut buf_reader = BufReader::new(bytes);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;
    Ok(find_links(&contents).iter().map(|link| link.to_string()).collect())
}

#[cfg(test)]
mod tests {
    use crate::formats::xml::extract_links;

    const TEST_XML: &[u8] = include_bytes!("../../assets/examples/xml/test.xml");
    #[test]
    fn get_some_website() {
        let links = extract_links(TEST_XML).unwrap();
        assert_eq!(links, vec!["http://www.placeholder-name-here.com/schema/"])
    }
}
