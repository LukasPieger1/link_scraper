use std::fmt::{Display, Formatter};
use std::io::{BufRead, BufReader};
use thiserror::Error;
use crate::gen_scrape_from_file;
use crate::link_scraper::find_urls;

pub fn scrape(bytes: &[u8]) -> Result<Vec<TextFileLink>, TextFileScrapingError> {
    let mut collector: Vec<TextFileLink> = vec![];
    let mut buf_reader = BufReader::new(bytes);
    let mut contents = String::new();
    let mut line_result = buf_reader.read_line(&mut contents)?;
    let mut current_line = 1;
    while line_result > 0 {
        find_urls(&contents)
            .iter()
            .for_each(|link| collector.push(TextFileLink {
                url: link.as_str().to_string(),
                location: TextFileLinkLocation { line: current_line, pos: link.start() }
            }));

        contents.clear();
        line_result = buf_reader.read_line(&mut contents)?;
        current_line += 1;
    }
    Ok(collector)
}
gen_scrape_from_file!(Result<Vec<TextFileLink>, TextFileScrapingError>);

#[derive(Error, Debug)]
pub enum TextFileScrapingError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

#[derive(Debug, Clone)]
pub struct TextFileLink {
    pub url: String,
    pub location: TextFileLinkLocation,
}

impl Display for TextFileLink {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.url)
    }
}

#[derive(Debug, Clone)]
pub struct TextFileLinkLocation {
    pub line: usize,
    pub pos: usize
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_XML: &[u8] = include_bytes!("../../test_files/xml/test.xml");

    #[test]
    fn scrape_text_file_test() {
        let links = scrape(TEST_XML).unwrap();
        println!("{:?}", links);
        assert_eq!(links.len(), 1)
    }
    #[test]
    fn scrape_text_file_from_file_test() {
        let links = scrape_from_file(Path::new("./test_files/xml/test.xml")).unwrap();
        println!("{:?}", links);
        assert_eq!(links.len(), 1)
    }
}