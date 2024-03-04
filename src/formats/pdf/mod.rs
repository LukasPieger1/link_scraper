mod pdf_operator;

use std::string::String;
use crate::link_extractor::find_links;
use lopdf::{Document, Object};
use thiserror::Error;
use std::str::{FromStr, Utf8Error};
use log::error;
use lopdf::content::Operation;
use crate::formats::pdf::pdf_operator::{get_operands_Tj, get_operands_TJ, I64OrString, PdfOperator};
use crate::formats::pdf::PdfExtractionError::UnterminatedTextObjectError;
use crate::formats::pdf::pdf_operator::PdfOperator::{TJ, Tj};

#[derive(Error, Debug)]
pub enum PdfExtractionError {
    #[error(transparent)]
    LopdfError(#[from] lopdf::Error),
    #[error(transparent)]
    StrumParseError(#[from] strum::ParseError),
    #[error("PDF contains an unterminated Text object.")]
    UnterminatedTextObjectError,
    #[error(transparent)]
    UTF8Error(#[from] Utf8Error),
}

#[derive(Debug)]
struct TextObject(Vec<Operation>);

impl TextObject {
    fn to_text(&self) -> Result<String, PdfExtractionError> {
        let mut str = String::new();
        for operation in &self.0 {
            let operator = PdfOperator::from_str(&operation.operator).map_err(PdfExtractionError::from)?;
            if operator == Tj || operator == TJ {
                let text;
                if operator == Tj {
                    text = get_operands_Tj(&operation.operands)
                } else {
                    text = get_operands_TJ(&operation.operands).iter()
                        .filter_map(|int_or_string| {
                            if let I64OrString::String(value) = int_or_string { Some(value.to_string()) }
                            else { None }
                        }).collect()
                }
                str += &text;
            }
        }
        Ok(str)
    }
}

/**
 * Tries to read the PDF at the given filePath and returns its contents as a String
**/
pub fn read_to_text(doc: Document) -> Result<String, PdfExtractionError> {
    let mut content = doc.page_iter()
        .flat_map(|page_id| doc.get_and_decode_page_content(page_id).unwrap().operations);

    let mut text_objects = vec![];

    while let Some(op) = content.next() {
        let operator = PdfOperator::from_str(&op.operator)?;

        // This is the start of a Text object
        if operator == PdfOperator::BT {
            text_objects.push(get_text_object(&mut content)?)
        }
    }

    for text in text_objects {
        // println!("{:?}", text);
        println!("{}", text.to_text()?);
    }
    Ok("unfinished".to_string())
}

fn get_text_object(content: &mut dyn Iterator<Item=Operation>) -> Result<TextObject, PdfExtractionError> {
    let mut new_text_object = vec![];
    for operation in content {
        if PdfOperator::from_str(&operation.operator)? == PdfOperator::ET {
            return Ok(TextObject(new_text_object));
        }
        new_text_object.push(operation);
    }
    Err(UnterminatedTextObjectError)
}

#[cfg(feature = "link_extraction")]
pub fn extract_links_simple(pdf: Document) -> Result<Vec<String>, PdfExtractionError> {
    let all_pages: Vec<u32> = pdf.page_iter().enumerate()
        .map(|(page_number, _page_object)| page_number as u32 + 1).collect();
    let plain_text = pdf.extract_text(&all_pages)?;
    // TODO get text from image-data as well?
    Ok(find_links(&plain_text).iter()
        .map(|it| it.to_string()).collect())
}

#[cfg(feature = "link_extraction")]
pub fn extract_links_simpler(input: &[u8]) -> Result<Vec<String>, PdfExtractionError> {
    let ret = find_links(&pdf_extract::extract_text_from_mem(input).unwrap())
        .iter()
        .map(|it| it.to_string())
        .collect();
    Ok(ret)
}

#[cfg(feature = "link_extraction")]
pub fn extract_links_from_byte_array(input: &[u8]) -> Result<Vec<String>, PdfExtractionError> {
    let pdf_doc = Document::load_mem(input)?;
    extract_links_simple(pdf_doc)
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;
    use std::include_bytes;

    // const TEST_PDF: &[u8]  = include_bytes!("../../../assets/examples/pdf/Studienbescheinigung.pdf");
    const TEST_PDF: &[u8] = include_bytes!("../../../assets/examples/pdf/Ticket-Uppsala-Goeteborg-3141969404.pdf");
    // const TEST_PDF: &[u8]  = include_bytes!("../../../assets/examples/pdf/2023-09-18_11-22-41.pdf");
    // const TEST_PDF: &[u8]  = include_bytes!("../../../assets/examples/pdf/2024-01-12_23-23-34.pdf");
    // const TEST_PDF: &[u8]  = include_bytes!("../../../assets/examples/pdf/eng.easyroam-App_Linux_Ubuntu_v22.pdf");
    // const TEST_PDF: &[u8]  = include_bytes!("../../../assets/examples/pdf/PDF32000_2008.pdf");

    #[test]
    fn read_to_text_test() {
        let doc = Document::load_mem(TEST_PDF).unwrap();
        let _ = read_to_text(doc);
    }

    #[test]
    fn extract_urls_from_pdf() {
        let doc = Document::load_mem(TEST_PDF).unwrap();
        println!(
            "{:?}",
            extract_links_simple(doc)
                .unwrap()
                .iter()
                .map(|url| url.to_string())
                .collect_vec()
        )
    }

    #[test]
    fn read_to_text_exploration() {
        _ = read_to_text(Document::load_mem(TEST_PDF).unwrap()).unwrap();
    }
}
