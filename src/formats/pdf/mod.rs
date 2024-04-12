use std::string::String;
use itertools::Itertools;
use thiserror::Error;
use mupdf::{Document, Page};
use crate::link_extractor::find_links;

#[derive(Error, Debug)]
pub enum PdfExtractionError {
    #[error(transparent)]
    MuPdfError(#[from] mupdf::Error),
    #[error("The file was encrypted, but no password was given.")]
    FileEncryptedError,
    #[error("You tried to decrypt a non-encrypted file.")]
    FileNotEncryptedError,
    #[error("Given file was not a PDF.")]
    NotAPdfError,
}

/// Reads a PDF as a bytearray and extracts all links from it.
///
/// For encrypted files please use [`extract_links_encrypted`] instead
pub fn extract_links(bytes: &[u8]) -> Result<Vec<String>, PdfExtractionError> {
    extract_links_from_doc(bytes_to_pdf(bytes)?)
}

/// Like [`extract_links`] for encrypted files.
/// Currently not working. Probably because of a bug in the mupdf package.
///
/// I created an <a href="https://github.com/messense/mupdf-rs/issues/82">issue</a> for it.
/// However, I don't think it is likely to be resolved.
pub fn extract_links_encrypted(bytes: &[u8], password: &str) -> Result<Vec<String>, PdfExtractionError> {
    let mut doc = bytes_to_pdf(bytes)?;
    if !doc.needs_password()? {
        return Err(PdfExtractionError::FileNotEncryptedError);
    }

    doc.authenticate(password)?;
    extract_links_from_doc(doc)
}

fn extract_links_from_doc(doc: Document) -> Result<Vec<String>, PdfExtractionError> {
    if !doc.is_pdf() {
        return Err(PdfExtractionError::NotAPdfError);
    }
    if doc.needs_password()? {
        return Err(PdfExtractionError::FileEncryptedError);
    }

    let mut links: Vec<String> = vec![];
    for page_res in doc.pages()? {
        let page = page_res?;
        find_text_links(&page, &mut links)?;
        find_hyperlinks(&page, &mut links)?;
    }

    links = links.iter()
        .unique()
        .map(|it| it.to_owned())
        .collect();

    Ok(links)
}

/// Finds plaintext links on a page
fn find_text_links(page: &Page, links: &mut Vec<String>) -> Result<(), PdfExtractionError> {
    find_links(&page.to_text()?).iter()
        .for_each(|link|
            links.push(link.to_string()));
    Ok(())
}

/// Finds hyperlinks on a page
fn find_hyperlinks(page: &Page, links: &mut Vec<String>) -> Result<(), PdfExtractionError> {
    for link in page.links()? {
        find_links(&link.uri).iter()
            .for_each(|link|
                links.push(link.to_string()));
    }
    Ok(())
}

fn bytes_to_pdf(bytes: &[u8]) -> Result<Document, PdfExtractionError> {
    Ok(Document::from_bytes(bytes, "test.pdf")?)
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::include_bytes;

    const NOT_A_PDF: &[u8] = include_bytes!("../../../test_files/docx/test.docx");
    const BIG_PDF: &[u8] = include_bytes!("../../../test_files/pdf/test.pdf");
    const PDFA_EXAMPLE: &[u8] = include_bytes!("../../../test_files/pdf/pdfa-example.pdf");
    const BIG_PDF_ENCRYPTED: &[u8] = include_bytes!("../../../test_files/pdf/test_protected.pdf"); // pass: asdfasdf

    #[test]
    fn extract_lots_of_links_from_pdf() {
        let links = extract_links(BIG_PDF).unwrap();
        assert_eq!(38, links.len())
    }

    #[test]
    fn extract_links_from_pdfa() {
        let links = extract_links(PDFA_EXAMPLE).unwrap();
        println!("{:?}", links);
        assert_eq!(links, vec!["http://www.tcpdf.org", "http://sourceforge.net/donate/index.php?group_id=128076"])
    }

    #[test]
    fn fail_on_encrypted_without_pw() {
        let links = extract_links(BIG_PDF_ENCRYPTED);
        assert!(links.is_err())
    }

    #[test]
    fn fail_on_decrypting_non_encrypted_file() {
        let links = extract_links_encrypted(BIG_PDF, "asdfasdf");
        assert!(links.is_err())
    }

    // /// Currently failing because of a mupdf bug (see [`extract_links_encrypted`])
    // #[test]
    // fn extract_lots_of_links_from_encrypted_pdf() {
    //     let links = extract_links_encrypted(BIG_PDF_ENCRYPTED, "asdfasdf").unwrap();
    //     assert_eq!(38, links.len())
    // }

    #[test]
    fn fail_on_non_pdf() {
        let error = bytes_to_pdf(NOT_A_PDF);
        assert!(error.is_err())
    }
}
