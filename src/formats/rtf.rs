use crate::gen_scrape_froms;
use crate::helpers::find_urls;
use itertools::Itertools;
use rtf_parser::lexer::Lexer;
use rtf_parser::tokens::Token;
use std::fmt::{Display, Formatter};
use std::io::{read_to_string, Read};
use thiserror::Error;

/// Limitations: Currently cannot extract Hyperlinks or comments.
/// But you may use [`formats::plaintext::scrape`] for those.
pub fn scrape<R>(reader: R) -> Result<Vec<RtfLink>, RtfScrapingError>
where
    R: Read,
{
    scrape_from_string(&read_to_string(reader)?)
}

/// Limitations: Currently cannot extract Hyperlinks or comments.
/// But you may use [`formats::plaintext::scrape`] for those.
pub fn scrape_from_string(s: &str) -> Result<Vec<RtfLink>, RtfScrapingError> {
    let tokens = Lexer::scan(s)?;
    let mut text = String::new();
    tokens.iter().for_each(|token| {
        if let Token::PlainText(pt) = token {
            text += pt;
            text += " "
        }
    });
    Ok(find_urls(&text)
        .iter()
        .map(|link| RtfLink {
            url: link.as_str().to_string(),
        })
        .collect_vec())
}

gen_scrape_froms!(scrape(Read) -> Result<Vec<RtfLink>, RtfScrapingError>);

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

    const TEST_RTF: &[u8] = include_bytes!("../../test_files/rtf/rtf_test.rtf");
    #[test]
    fn scrape_rtf_test() {
        let links = scrape(TEST_RTF).unwrap();
        println!("{:?}", links);
        assert!(links
            .iter()
            .any(|it| it.url == "https://plaintext.test.com"));
    }
}
