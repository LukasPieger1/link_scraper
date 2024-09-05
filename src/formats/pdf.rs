use std::fmt::{Display, Formatter};
use std::string::String;
use thiserror::Error;
use mupdf::{Document, Page};
use crate::gen_scrape_from_file;
use crate::helpers::find_urls;

/// Reads a PDF as a bytearray and scrapes all links from it.
///
/// For encrypted files please use [`scrape_encrypted`] instead
pub fn scrape(bytes: &[u8]) -> Result<Vec<PdfLink>, PdfScrapingError> {
    scrape_from_doc(bytes_to_pdf(bytes)?)
}
gen_scrape_from_file!(Result<Vec<PdfLink>, PdfScrapingError>);

#[derive(Error, Debug)]
pub enum PdfScrapingError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    MuPdfError(#[from] mupdf::Error),
    #[error("The file was encrypted, but no password was given.")]
    FileEncryptedError,
    #[error("You tried to decrypt a non-encrypted file.")]
    FileNotEncryptedError,
    #[error("Given file was not a PDF.")]
    NotAPdfError,
}

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PdfLinkLocation {
    pub page: usize
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PdfLinkKind {
    PlainText,
    Hyperlink
}

/// Like [`scrape`] for encrypted files.
/// Currently not working. Probably because of a bug in the mupdf package.
///
/// I created an <a href="https://github.com/messense/mupdf-rs/issues/82">issue</a> for it.
/// However, I don't think it is likely to be resolved.
pub fn scrape_encrypted(bytes: &[u8], password: &str) -> Result<Vec<PdfLink>, PdfScrapingError> {
    let mut doc = bytes_to_pdf(bytes)?;
    if !doc.needs_password()? {
        return Err(PdfScrapingError::FileNotEncryptedError);
    }

    doc.authenticate(password)?;
    scrape_from_doc(doc)
}

fn scrape_from_doc(doc: Document) -> Result<Vec<PdfLink>, PdfScrapingError> {
    if !doc.is_pdf() {
        return Err(PdfScrapingError::NotAPdfError);
    }
    if doc.needs_password()? {
        return Err(PdfScrapingError::FileEncryptedError);
    }

    let mut links: Vec<PdfLink> = vec![];
    let mut page_number = 1;
    for page_res in doc.pages()? {
        let page = page_res?;
        find_text_links(&page, page_number, &mut links)?;
        find_hyperlinks(&page, page_number, &mut links)?;
        page_number += 1
    }

    Ok(links)
}

/// Finds plaintext links on a page
fn find_text_links(page: &Page, page_number: usize, links: &mut Vec<PdfLink>) -> Result<(), PdfScrapingError> {
    find_urls(&page.to_text()?).iter()
        .for_each(|link|
            links.push(PdfLink {
                url: link.as_str().to_string(),
                location: PdfLinkLocation { page: page_number },
                kind: PdfLinkKind::PlainText,
            }));
    Ok(())
}

/// Finds hyperlinks on a page
fn find_hyperlinks(page: &Page, page_number: usize, links: &mut Vec<PdfLink>) -> Result<(), PdfScrapingError> {
    for link in page.links()? {
        find_urls(&link.uri).iter()
            .for_each(|link|
                links.push(PdfLink { 
                    url: link.as_str().to_string(),
                    location: PdfLinkLocation { page: page_number },
                    kind: PdfLinkKind::Hyperlink,
                }));
    }
    Ok(())
}

fn bytes_to_pdf(bytes: &[u8]) -> Result<Document, PdfScrapingError> {
    Ok(Document::from_bytes(bytes, "file.pdf")?)
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::include_bytes;

    const NOT_A_PDF: &[u8] = include_bytes!("../../test_files/ooxml/xlsx_test.xlsx");
    const TEST_PDF: &[u8] = include_bytes!("../../test_files/pdf/pdf_test.pdf");
    const TEST_PDFA: &[u8] = include_bytes!("../../test_files/pdf/pdfa_test.pdf");
    const TEST_PDF_ENCRYPTED: &[u8] = include_bytes!("../../test_files/pdf/pdf_protected_test.pdf"); // pass: asdfasdf

    #[test]
    fn scrape_pdf_test() {
        let links = scrape(TEST_PDF).unwrap();
        println!("{:?}", links);
        assert!(links.iter().any(|it| it.url == "https://hyperlink.test.com/" && it.kind == PdfLinkKind::Hyperlink));
        assert!(links.iter().any(|it| it.url == "https://plaintext.test.com" && it.kind == PdfLinkKind::PlainText));
    }

    #[test]
    fn scrape_pdfa_test() {
        let links = scrape(TEST_PDFA).unwrap();
        println!("{:?}", links);
        assert!(links.iter().any(|it| it.url == "https://hyperlink.test.com/" && it.kind == PdfLinkKind::Hyperlink));
        assert!(links.iter().any(|it| it.url == "https://plaintext.test.com" && it.kind == PdfLinkKind::PlainText));
    }

    #[test]
    fn fail_on_encrypted_without_pw_test() {
        let links = scrape(TEST_PDF_ENCRYPTED);
        println!("{:?}", links);
        assert!(links.is_err())
    }

    #[test]
    fn fail_on_decrypting_non_encrypted_file_test() {
        let links = scrape_encrypted(TEST_PDF, "asdfasdf");
        println!("{:?}", links);
        assert!(links.is_err())
    }

    // /// Currently failing because of a mupdf bug (see [`scrape_encrypted`])
    // #[test]
    // fn scrape_lots_from_pdf_test() {
    //     let links = scrape_encrypted(BIG_PDF_ENCRYPTED, "asdfasdf").unwrap();
    //     assert_eq!(38, links.len())
    // }

    #[test]
    fn fail_on_non_pdf_test() {
        let error = bytes_to_pdf(NOT_A_PDF);
        assert!(error.is_err())
    }
}
