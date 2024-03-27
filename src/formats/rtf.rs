use std::io::{read_to_string};

use itertools::Itertools;
use rtf_parser::lexer::Lexer;
use rtf_parser::tokens::Token;
use thiserror::Error;

use crate::link_extractor::find_links;

#[derive(Error, Debug)]
pub enum RtfExtractionError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    LexerError(#[from] rtf_parser::lexer::LexerError),
    #[error(transparent)]
    ParserError(#[from] rtf_parser::parser::ParserError),
}

pub fn extract_links(bytes: &[u8]) -> Result<Vec<String>, RtfExtractionError> {
    let data = read_to_string(bytes)?;
    let tokens = Lexer::scan(&data)?;
    let mut text = String::new();
    tokens.iter().for_each(|token| if let Token::PlainText(pt) = token {text += pt; text += " "});
    // let mut document = Parser::new(tokens);
    // let text = document.parse()?.get_text();
    Ok(find_links(&text).iter().map(|link| link.to_string()).collect_vec())
}

#[cfg(test)]
mod tests {
    use crate::link_extractor::unique_and_sort;
    use super::*;

    const TEST_RTF: &[u8] = include_bytes!("../../assets/examples/rtf/file-sample_1MB.rtf");
    #[test]
    fn extract_links_from_rtf() {
        let links = extract_links(TEST_RTF).unwrap();
        assert_eq!(unique_and_sort(&links), vec!["http://test.plain.ru", "http://test.plain.ru/", "https://lol.brot.tv/", "https://products.office.com/en-us/word"])
    }
}
