use std::fmt::{Display, Formatter};
use std::string::String;
use thiserror::Error;
use mupdf::{Document, Page};
use crate::link_extractor::find_urls;

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

#[derive(Debug, Clone)]
pub struct PdfLink {
    pub url: String,
    pub location: PdfLinkLocation,
    pub kind: PdfLinkKind,
}

impl Display for PdfLink {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.url)
    }
}

#[derive(Debug, Clone)]
pub struct PdfLinkLocation {
    pub page: usize
}

#[derive(Debug, Clone, Copy)]
pub enum PdfLinkKind {
    PlainText,
    Hyperlink
    //TODO: Comment
}

/// Reads a PDF as a bytearray and extracts all links from it.
///
/// For encrypted files please use [`extract_links_encrypted`] instead
pub fn extract_links(bytes: &[u8]) -> Result<Vec<PdfLink>, PdfExtractionError> {
    extract_links_from_doc(bytes_to_pdf(bytes)?)
}

/// Like [`extract_links`] for encrypted files.
/// Currently not working. Probably because of a bug in the mupdf package.
///
/// I created an <a href="https://github.com/messense/mupdf-rs/issues/82">issue</a> for it.
/// However, I don't think it is likely to be resolved.
pub fn extract_links_encrypted(bytes: &[u8], password: &str) -> Result<Vec<PdfLink>, PdfExtractionError> {
    let mut doc = bytes_to_pdf(bytes)?;
    if !doc.needs_password()? {
        return Err(PdfExtractionError::FileNotEncryptedError);
    }

    doc.authenticate(password)?;
    extract_links_from_doc(doc)
}

fn extract_links_from_doc(doc: Document) -> Result<Vec<PdfLink>, PdfExtractionError> {
    if !doc.is_pdf() {
        return Err(PdfExtractionError::NotAPdfError);
    }
    if doc.needs_password()? {
        return Err(PdfExtractionError::FileEncryptedError);
    }

    let mut links: Vec<PdfLink> = vec![];
    for page_res in doc.pages()? {
        let page = page_res?;
        find_text_links(&page, &mut links)?;
        find_hyperlinks(&page, &mut links)?;
    }

    Ok(links)
}

/// Finds plaintext links on a page
fn find_text_links(page: &Page, links: &mut Vec<PdfLink>) -> Result<(), PdfExtractionError> {
    find_urls(&page.to_text()?).iter()
        .for_each(|link|
            links.push(PdfLink {
                url: link.as_str().to_string(),
                location: PdfLinkLocation { page: 0 },//TODO actually assign page
                kind: PdfLinkKind::Hyperlink,
            }));
    Ok(())
}

/// Finds hyperlinks on a page
fn find_hyperlinks(page: &Page, links: &mut Vec<PdfLink>) -> Result<(), PdfExtractionError> {
    for link in page.links()? {
        find_urls(&link.uri).iter()
            .for_each(|link|
                links.push(PdfLink { 
                    url: link.as_str().to_string(),
                    location: PdfLinkLocation { page: 0 },//TODO actually assign page
                    kind: PdfLinkKind::PlainText,
                }));
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

    const NOT_A_PDF: &[u8] = include_bytes!("../../test_files/docx/test.docx");
    const BIG_PDF: &[u8] = include_bytes!("../../test_files/pdf/test.pdf");
    const PDFA_EXAMPLE: &[u8] = include_bytes!("../../test_files/pdf/pdfa-example.pdf");
    const BIG_PDF_ENCRYPTED: &[u8] = include_bytes!("../../test_files/pdf/test_protected.pdf"); // pass: asdfasdf

    #[test]
    fn extract_lots_of_links_from_pdf() {
        let links = extract_links(BIG_PDF).unwrap();
        assert_eq!(38, links.len())
    }

    #[test]
    fn extract_links_from_pdfa() {
        let links = extract_links(PDFA_EXAMPLE).unwrap();
        assert_eq!(links.len(), 2)
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
