use std::string::String;
use itertools::Itertools;
use thiserror::Error;
use mupdf::{Document, Page};
use crate::link_extractor::find_links;

#[derive(Error, Debug)]
pub enum PdfExtractionError {
    #[error(transparent)]
    MuPdfError(#[from] mupdf::Error)
}

fn find_text_links(page: &Page) -> Result<Vec<String>, PdfExtractionError> {
    Ok(
        find_links(&page.to_text()?).iter()
            .map(|link| link.to_string()).collect()
    )
}

fn find_hyperlinks(page: &Page) -> Result<Vec<String>, PdfExtractionError> {
    let mut links: Vec<String> = vec![];
    for link in page.links()? {
        find_links(&link.uri).iter()
            .map(|link| link.to_string())
            .for_each(|link| links.push(link));
    }

    Ok(links)
}

pub fn extract_links(doc: Document) -> Result<Vec<String>, PdfExtractionError> {
    let mut links: Vec<String> = vec![];
    for page_res in doc.pages()? {
        let page = page_res?;
        links.append(&mut find_text_links(&page)?);
        links.append(&mut find_hyperlinks(&page)?);
    }

    links = links.iter()
        .unique()
        .map(|it| it.to_owned())
        .collect();

    Ok(links)
}



#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;
    use std::include_bytes;

    const TEST_PDF: &[u8]  = include_bytes!("../../../assets/examples/pdf/Studienbescheinigung.pdf");
    // const TEST_PDF: &[u8] = include_bytes!("../../../assets/examples/pdf/Ticket-Uppsala-Goeteborg-3141969404.pdf");
    // const TEST_PDF: &[u8]  = include_bytes!("../../../assets/examples/pdf/2023-09-18_11-22-41.pdf");
    // const TEST_PDF: &[u8]  = include_bytes!("../../../assets/examples/pdf/2024-01-12_23-23-34.pdf");
    // const TEST_PDF: &[u8]  = include_bytes!("../../../assets/examples/pdf/eng.easyroam-App_Linux_Ubuntu_v22.pdf");
    // const TEST_PDF: &[u8]  = include_bytes!("../../../assets/examples/pdf/PDF32000_2008.pdf");

    #[test]
    fn extract_links_from_pdf() {
        let doc = Document::from_bytes(TEST_PDF, "").unwrap();
        println!("{:?}", extract_links(doc).unwrap().iter()
                .map(|url| url.to_string()).collect_vec()
        )
    }
}
