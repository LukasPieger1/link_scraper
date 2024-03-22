use std::fmt::Pointer;
use std::io::Read;
use docx_rs::{DocumentChild, Docx, TableCell, TableChild, TableRowChild};
use dotext::MsDoc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DocxExtractionError {
    #[error(transparent)]
    ReaderError(#[from] std::io::Error),
}

fn extract_hyperlinks(docx: &Docx, collector: &mut Vec<String>) {
    docx.hyperlinks.iter()
        .filter(|(_,_,hyperlink_type)| hyperlink_type=="External")
        .for_each(|(_,hyperlink_ref,_)| collector.push(hyperlink_ref.to_string()))
}

fn extract_comment_links(docx: &Docx, collector: &mut Vec<String>) {
    // docx.comments_extended.children.iter()
    //     .filter(|comment| comment.)
    //     .for_each(|(_,hyperlink_ref,_)| collector.push(hyperlink_ref.to_string()))
}

fn extract_plaintext_links(docx: &Docx, collector: &mut Vec<String>) {
    // docx.document.children.iter()
    //     .map(|child| get_text_out_of_that_piece_of_shit(child))
}

pub fn extract_links(bytes: &mut [u8]) -> Result<Vec<String>, DocxExtractionError> {
    let docx = docx_rs::read_docx(bytes).unwrap();
    
    let mut links = vec![];
    extract_hyperlinks(&docx, &mut links);
    extract_plaintext_links(&docx, &mut links);

    Ok(links)
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::include_bytes;
    use std::path::Path;

    const TEST_DOCX: &[u8] = include_bytes!("../../assets/examples/docx/demo.docx");
    
    #[test]
    pub fn extract_links_from_docx() {
        let mut striiing = String::new();
        dotext::docx::Docx::open(Path::new("assets/examples/docx/demo.docx")).unwrap().read_to_string(&mut striiing).unwrap();
        println!("{}", striiing);

        // let links = extract_links(TEST_DOCX).unwrap();
        // assert_eq!(links, vec!["http://www.placeholder-name-here.com/schema/"])
    }
}
