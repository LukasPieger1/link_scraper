use std::fmt::{Display, Formatter};
use std::io::read_to_string;

use itertools::Itertools;
use rtf_parser::lexer::Lexer;
use rtf_parser::tokens::Token;
use thiserror::Error;
use xml::common::TextPosition;

use crate::link_extractor::find_urls;

#[derive(Error, Debug)]
pub enum RtfExtractionError {
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
    pub location: usize,
    //TODO make location less useless
}

impl Display for RtfLink {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.url)
    }
}

#[derive(Debug, Clone)]
pub struct RtfLinkLocation {
    pub file: String,
    pub position: TextPosition
}

pub fn extract_links(bytes: &[u8]) -> Result<Vec<RtfLink>, RtfExtractionError> {
    let data = read_to_string(bytes)?;
    let tokens = Lexer::scan(&data)?;
    let mut text = String::new();
    tokens.iter().for_each(|token| if let Token::PlainText(pt) = token {text += pt; text += " "});
    Ok(find_urls(&text).iter().map(|link| RtfLink {
        url: link.as_str().to_string(),
        location: link.start()
    }).collect_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_RTF: &[u8] = include_bytes!("../../test_files/rtf/test.rtf");
    #[test]
    fn extract_links_from_rtf() {
        let links = extract_links(TEST_RTF).unwrap();
        assert_eq!(links.len(), 4)
    }
}
