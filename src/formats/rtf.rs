use std::fmt::{Display, Formatter};
use std::io::read_to_string;
use itertools::Itertools;
use rtf_parser::lexer::Lexer;
use rtf_parser::tokens::Token;
use thiserror::Error;
use crate::gen_scrape_from_file;
use crate::link_scraper::find_urls;

pub fn scrape(bytes: &[u8]) -> Result<Vec<RtfLink>, RtfScrapingError> {
    let data = read_to_string(bytes)?;
    let tokens = Lexer::scan(&data)?;
    let mut text = String::new();
    tokens.iter().for_each(|token| if let Token::PlainText(pt) = token {text += pt; text += " "});
    Ok(find_urls(&text).iter().map(|link| RtfLink {
        url: link.as_str().to_string(),
    }).collect_vec())
}
gen_scrape_from_file!(Result<Vec<RtfLink>, RtfScrapingError>);

#[derive(Error, Debug)]
pub enum RtfScrapingError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    LexerError(#[from] rtf_parser::lexer::LexerError),
    #[error(transparent)]
    ParserError(#[from] rtf_parser::parser::ParserError),
}

#[derive(Debug, Clone)]
pub struct RtfLink {
    pub url: String,
}

impl Display for RtfLink {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_RTF: &[u8] = include_bytes!("../../test_files/rtf/test.rtf");
    #[test]
    fn scrape_rtf_test() {
        let links = scrape(TEST_RTF).unwrap();
        assert_eq!(links.len(), 4)
    }
}
