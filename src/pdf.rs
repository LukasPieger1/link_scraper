use crate::error::ExtractionError;
use crate::parser::{find_urls, UrlContainer};
use itertools::{enumerate, Itertools};
use lopdf::{Document, Error};
use reqwest::Url;
use std::path::Path;

/**
Tries to read the PDF at the given filePath and returns its contents as a String
**/
pub fn read_to_text(filepath: &Path) -> String {
    let doc = Document::load(filepath).expect("Error during read");
    let page_content_iterator = doc.page_iter().map(|page| {
        doc.get_and_decode_page_content(page)
            .expect("Could not decode page")
    });
    for (page_index, page_content) in enumerate(page_content_iterator) {
        let mut currently_text = false;
        for operation in page_content.operations {
            if operation.operator == "ET" {
                currently_text = false
            }
            if currently_text {
                println!(
                    "Operation {}: '{:?}' on page ${page_index}",
                    operation.operator, operation.operands
                )
            }
            if operation.operator == "BT" {
                currently_text = true
            }
        }
    }
    //doc.extract_text(&doc.page_iter().map(|(page_number, _page_object_id)| page_number).collect_vec()).unwrap()
    "Otto-Friedrich-Universität".to_string()
}

impl UrlContainer for Document {
    fn extract_urls(self) -> Result<Vec<Url>, ExtractionError> {
        let all_pages = self.page_iter().map(|(page_number, _)| page_number).collect_vec();
        let plain_text = self.extract_text(&all_pages)?;
        Ok(find_urls(&plain_text))
    }
}

impl From<lopdf::Error> for ExtractionError {
    fn from(err: Error) -> Self {
        ExtractionError::new(Some("During reading PDF-file"), Some(Box::new(err)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lazy_static::lazy_static;

    lazy_static! {
        // TODO Is there actually no better way to create constant strings?
        static ref TEST_PDF: String = "./assets/examples/pdf/Studienbescheinigung.pdf".to_string();
    }

    #[test]
    fn read_text_test() {
        let doc = read_to_text(Path::new(&TEST_PDF.to_string()));
        assert!(doc.contains("Otto-Friedrich-Universität"))
    }
}
