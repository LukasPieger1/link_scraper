use crate::gen_scrape_froms;
use crate::helpers::find_urls;
use itertools::Itertools;
use rtf_parser::lexer::Lexer;
use rtf_parser::tokens::Token;
use std::fmt::{Display, Formatter};
use std::io::{BufRead, Read};
use thiserror::Error;

/// Limitations: Currently cannot extract Hyperlinks or comments.
/// But you may use [`formats::plaintext::scrape`] for those.
///
/// Reads the whole stream before processing the contents and converts it to str.
/// Use [`scrape_from_string`] to omit the [`BufRead`].
pub fn scrape<R>(mut reader: R) -> Result<Vec<RtfLink>, RtfScrapingError>
where
    R: BufRead,
{
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;
    scrape_from_slice(buffer)
}

/// Limitations: Currently cannot extract Hyperlinks or comments.
/// But you may use [`formats::plaintext::scrape`] for those.
pub fn scrape_from_string<S>(s: S) -> Result<Vec<RtfLink>, RtfScrapingError>
where
    S: AsRef<str>,
{
    let tokens = Lexer::scan(s.as_ref())?;
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

/// Limitations: Currently cannot extract Hyperlinks or comments.
/// But you may use [`formats::plaintext::scrape`] for those.
///
/// Returns an error if `bytes` is not valid in UTF-8
pub fn scrape_from_slice<T>(bytes: T) -> Result<Vec<RtfLink>, RtfScrapingError>
where
    T: AsRef<[u8]>,
{
    scrape_from_string(std::str::from_utf8(bytes.as_ref())?)
}

gen_scrape_froms!(scrape_from_slice(AsRef<[u8]>) -> Result<Vec<RtfLink>, RtfScrapingError>);

#[derive(Error, Debug)]
pub enum RtfScrapingError {
    #[error(transparent)]
    Utf8Error(#[from] std::str::Utf8Error),
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
        let links = scrape_from_slice(TEST_RTF).unwrap();
        println!("{:?}", links);
        assert!(links
            .iter()
            .any(|it| it.url == "https://plaintext.test.com"));
    }
}
