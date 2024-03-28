use std::io::read_to_string;
use thiserror::Error;
use crate::formats::odt::OdtExtractionError;
use crate::formats::ooxml::OoxmlExtractionError;
use crate::formats::pdf::PdfExtractionError;
use crate::formats::raw_text::TextFileExtractionError;
use crate::formats::rtf::RtfExtractionError;
use crate::link_extractor::find_links;

#[derive(Error, Debug)]
pub enum LinkExtractionError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[cfg(feature = "ooxml")]
    #[error(transparent)]
    OoxmlExtractionError(#[from] OoxmlExtractionError),

    #[cfg(feature = "odt")]
    #[error(transparent)]
    OdtExtractionError(#[from] OdtExtractionError),

    #[cfg(feature = "pdf")]
    #[error(transparent)]
    PdfExtractionError(#[from] PdfExtractionError),

    #[cfg(feature = "rtf")]
    #[error(transparent)]
    RtfExtractionError(#[from] RtfExtractionError),

    #[cfg(feature = "rtf")]
    #[error(transparent)]
    TextFileExtractionError(#[from] TextFileExtractionError),

    #[error("Required feature is not enabled")]
    FeatureNotEnabledError(String),

    #[error("Filetype not recognized")]
    FileTypeNotImplemented(String),

    #[error("Extraction failed")]
    ExtractionFailed(String)
}

/// Guesses the file-type and extracts links from the file.
pub fn extract_links(bytes: &[u8]) -> Result<Vec<String>, LinkExtractionError>{
    let file_type = infer::get(&bytes);

    if file_type == None {
        return Ok(find_links(&read_to_string(bytes)?).iter().map(|link| link.to_string()).collect());
    }
    let file_type = file_type.unwrap();

    match file_type.mime_type() {
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" => Ok(try_ooxml(bytes)?),
        "application/vnd.oasis.opendocument.text" => Ok(try_odt(bytes)?),
        "application/zip" => try_zip(bytes),
        "application/pdf" => Ok(try_pdf(bytes)?),
        "application/rtf" => Ok(try_rtf(bytes)?),
        "application/json" => Ok(try_text_file(bytes)?),
        "text/xml" => Ok(try_text_file(bytes)?),
        _ => Err(LinkExtractionError::FileTypeNotImplemented(file_type.mime_type().to_string()))
    }
}

fn try_zip(bytes: &[u8]) -> Result<Vec<String>, LinkExtractionError> {
    #[allow(unused_assignments)]
        let mut ret = Err(LinkExtractionError::FeatureNotEnabledError("Zip-file detected. Would try to parse it with `ooxml`/`odt`-feature but none of them is enabled. Please enable it in your dependencies.".to_string()));
    #[cfg(feature = "ooxml")] {
        ret = try_ooxml(bytes).map_err(|e| LinkExtractionError::from(e));
        if let Ok(res) = ret { return Ok(res) }
    }
    #[cfg(feature = "odt")] {
        ret = try_odt(bytes).map_err(|e| LinkExtractionError::from(e));
        if let Ok(res) = ret { return Ok(res) }
    }

    return ret;
}

fn try_text_file(bytes: &[u8]) -> Result<Vec<String>, TextFileExtractionError> {
    #[cfg(feature = "raw_text")]
    return Ok(crate::formats::raw_text::extract_links(bytes)?);
    #[cfg(not(feature = "raw_text"))]
    return Err(LinkExtractionError::FeatureNotEnabledError("text-document detected, but cannot parse it because `text_file`-feature is not enabled. Please enable it in your dependencies.".to_string()))
}

fn try_odt(bytes: &[u8]) -> Result<Vec<String>, OdtExtractionError> {
    #[cfg(feature = "odt")]
    return Ok(crate::formats::odt::extract_links(bytes)?);
    #[cfg(not(feature = "odt"))]
    return Err(LinkExtractionError::FeatureNotEnabledError("OpenOffice document detected, but cannot parse it because `odt`-feature is not enabled. Please enable it in your dependencies.".to_string()))
}

fn try_rtf(bytes: &[u8]) -> Result<Vec<String>, RtfExtractionError> {
    #[cfg(feature = "rtf")]
    return Ok(crate::formats::rtf::extract_links(bytes)?);
    #[cfg(not(feature = "rtf"))]
    return Err(LinkExtractionError::FeatureNotEnabledError(".rtf-document detected, but cannot parse it because `rtf`-feature is not enabled. Please enable it in your dependencies.".to_string()))
}

fn try_pdf(bytes: &[u8]) -> Result<Vec<String>, PdfExtractionError> {
    #[cfg(feature = "pdf")]
    return Ok(crate::formats::pdf::extract_links(bytes)?);
    #[cfg(not(feature = "pdf"))]
    return Err(LinkExtractionError::FeatureNotEnabledError("PDF-document detected, but cannot parse it because `pdf`-feature is not enabled. Please enable it in your dependencies.".to_string()))
}

fn try_ooxml(bytes: &[u8]) -> Result<Vec<String>, OoxmlExtractionError> {
    #[cfg(feature = "ooxml")]
    return Ok(crate::formats::ooxml::extract_links(bytes)?);
    #[cfg(not(feature = "ooxml"))]
    return Err(LinkExtractionError::FeatureNotEnabledError("Microsoft-office document detected, but cannot parse it because `ooxml`-feature is not enabled. Please enable it in your dependencies.".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::include_bytes;

    const TEST_DOCX: &[u8] = include_bytes!("../assets/examples/docx/demo.docx");
    const TEST_PPTX: &[u8] = include_bytes!("../assets/examples/pptx/samplepptx.pptx");
    const TEST_XLSX: &[u8] = include_bytes!("../assets/examples/xlsx/sample2.xlsx");
    const TEST_ODT: &[u8] = include_bytes!("../assets/examples/odt/file-sample_1MB.odt");
    const TEST_PDF: &[u8] = include_bytes!("../assets/examples/pdf/combined_test.pdf");
    const TEST_PDF2: &[u8] = include_bytes!("../assets/examples/pdf/PDF32000_2008.pdf");
    const TEST_RTF: &[u8] = include_bytes!("../assets/examples/rtf/file-sample_1MB.rtf");
    const TEST_XML: &[u8] = include_bytes!("../assets/examples/xml/test.xml");

    #[test]
    fn generic_extraction_test() {
        println!("{:?}", extract_links(b"http://test.com").unwrap());
        println!("{:?}", extract_links(TEST_DOCX).unwrap());
        println!("{:?}", extract_links(TEST_PPTX).unwrap());
        println!("{:?}", extract_links(TEST_XLSX).unwrap());
        println!("{:?}", extract_links(TEST_ODT).unwrap());
        println!("{:?}", extract_links(TEST_PDF).unwrap());
        println!("{:?}", extract_links(TEST_PDF2).unwrap());
        println!("{:?}", extract_links(TEST_RTF).unwrap());
        println!("{:?}", extract_links(TEST_XML).unwrap());
    }
}