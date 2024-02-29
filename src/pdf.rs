use crate::link_extractor::{find_urls};
use itertools::Itertools;
use lopdf::{Document, Object};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PdfExtractionError {
    #[error(transparent)]
    LopdfError(#[from] lopdf::Error)
}

/**
 * Tries to read the PDF at the given filePath and returns its contents as a String
**/
pub fn read_to_text(doc: Document) -> String {
    let content_iterator = doc.page_iter().flat_map(|page| doc.get_page_contents(page));

    // let mut text_content = vec![];
    let mut currently_text = false;
    for content_id in content_iterator {
        let operation: &Object = doc.get_object(content_id).unwrap();
        // if operation.operator == "ET" {
        //     currently_text = false
        // }
        // if currently_text {
        //     text_content.push(operation);
        //     continue;
        // }
        // if operation.operator == "BT" {
        //     currently_text = true
        // }
    }

    // TODO parse text-content
    //doc.extract_text(&doc.page_iter().map(|(page_number, _page_object_id)| page_number).collect_vec()).unwrap()
    "unfinished".to_string()
}

#[cfg(feature = "link_extraction")]
pub fn extract_urls_new(pdf: &[u8]) -> Result<Vec<String>, PdfExtractionError> {
    let out = pdf_extract::extract_text_from_mem(pdf).unwrap();
    Ok(find_urls(&out).iter().map(|it| it.to_string()).collect())
}

#[cfg(feature = "link_extraction")]
pub fn extract_urls(pdf: Document) -> Result<Vec<String>, PdfExtractionError> {
    let all_pages:Vec<u32> = pdf.page_iter().enumerate()
        .map(|(page_number, _page_object)| page_number as u32 + 1)
        .collect();
    let plain_text = pdf.extract_text(&all_pages)?;
    // TODO get text from image-data as well?
    Ok(find_urls(&plain_text).iter().map(|it| it.to_string()).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::include_bytes;

    const TEST_PDF: &[u8]  = include_bytes!("../assets/examples/pdf/Studienbescheinigung.pdf");
    // const TEST_PDF: &[u8]  = include_bytes!("../assets/examples/pdf/Ticket-Uppsala-Goeteborg-3141969404.pdf");
    // const TEST_PDF: &[u8]  = include_bytes!("../assets/examples/pdf/2023-09-18_11-22-41.pdf");
    // const TEST_PDF: &[u8]  = include_bytes!("../assets/examples/pdf/2024-01-12_23-23-34.pdf");
    // const TEST_PDF: &[u8]  = include_bytes!("../assets/examples/pdf/eng.easyroam-App_Linux_Ubuntu_v22.pdf");
    // const TEST_PDF: &[u8]  = include_bytes!("../assets/examples/pdf/PDF32000_2008.pdf");

    #[test]
    fn read_to_text_test() {
        let doc = Document::load_mem(TEST_PDF).unwrap();
        let _ = read_to_text(doc);
    }

    #[test]
    fn extract_urls_from_pdf() {
        let doc = Document::load_mem(TEST_PDF).unwrap();
        println!("{:?}", extract_urls(doc)
                .unwrap()
                .iter()
                .map(|url| url.to_string())
                .collect_vec()
        )
    }

    #[test]
    fn extract_urls_from_pdf_new() {
        println!("{:?}", extract_urls_new(TEST_PDF)
            .unwrap()
            .iter()
            .map(|url| url.to_string())
            .collect_vec()
        )
    }
}
