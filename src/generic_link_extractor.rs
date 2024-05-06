use std::io::read_to_string;
use thiserror::Error;
use crate::link_extractor::find_urls;

#[derive(Error, Debug)]
pub enum LinkExtractionError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[cfg(feature = "ooxml")]
    #[error(transparent)]
    OoxmlExtractionError(#[from] crate::formats::ooxml::OoxmlExtractionError),

    #[cfg(feature = "odf")]
    #[error(transparent)]
    OdtExtractionError(#[from] crate::formats::odf::OdfExtractionError),

    #[cfg(feature = "pdf")]
    #[error(transparent)]
    PdfExtractionError(#[from] crate::formats::pdf::PdfExtractionError),

    #[cfg(feature = "rtf")]
    #[error(transparent)]
    RtfExtractionError(#[from] crate::formats::rtf::RtfExtractionError),

    #[cfg(feature = "text_file")]
    #[error(transparent)]
    TextFileExtractionError(#[from] crate::formats::text_file::TextFileExtractionError),

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
        return Ok(find_urls(&read_to_string(bytes)?).iter().map(|link| link.to_string()).collect());
    }
    let file_type = file_type.unwrap();

    match file_type.mime_type() {
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" => Ok(try_ooxml(bytes)?),
        "application/vnd.oasis.opendocument.text" => Ok(try_odf(bytes)?),
        "application/vnd.oasis.opendocument.spreadsheet" => Ok(try_odf(bytes)?),
        "application/vnd.oasis.opendocument.template" => Ok(try_odf(bytes)?),
        "application/vnd.oasis.opendocument.presentation" => Ok(try_odf(bytes)?),
        "application/zip" => try_zip(bytes),
        "application/pdf" => Ok(try_pdf(bytes)?),
        "application/rtf" => Ok(try_rtf(bytes)?),

        "text/plain" => Ok(try_text_file(bytes)?),
        "text/xml" => Ok(try_text_file(bytes)?),
        "text/csv" => Ok(try_text_file(bytes)?),
        "text/html" => Ok(try_text_file(bytes)?),
        "text/css" => Ok(try_text_file(bytes)?),
        "application/json" => Ok(try_text_file(bytes)?),
        _ => Err(LinkExtractionError::FileTypeNotImplemented(file_type.mime_type().to_string()))
    }
}

fn try_zip(bytes: &[u8]) -> Result<Vec<String>, LinkExtractionError> {
    #[allow(unused_assignments)]
        let mut ret = Err(LinkExtractionError::FeatureNotEnabledError("Zip-file detected. Would try to parse it with `ooxml`/`odf`-feature but none of them is enabled. Please enable it in your dependencies.".to_string()));
    #[cfg(feature = "ooxml")] {
        ret = try_ooxml(bytes).map_err(|e| LinkExtractionError::from(e));
        if let Ok(res) = ret { return Ok(res) }
    }
    #[cfg(feature = "odf")] {
        ret = try_odf(bytes).map_err(|e| LinkExtractionError::from(e));
        if let Ok(res) = ret { return Ok(res) }
    }

    return ret;
}

fn try_text_file(bytes: &[u8]) -> Result<Vec<String>, LinkExtractionError> {
    #[cfg(feature = "text_file")]
    return Ok(crate::formats::text_file::extract_links(bytes)?);
    #[cfg(not(feature = "text_file"))]
    return Err(LinkExtractionError::FeatureNotEnabledError("text-document detected, but cannot parse it because `text_file`-feature is not enabled. Please enable it in your dependencies.".to_string()))
}

fn try_odf(bytes: &[u8]) -> Result<Vec<String>, LinkExtractionError> {
    #[cfg(feature = "odf")]
    return Ok(crate::formats::odf::extract_links(bytes)?);
    #[cfg(not(feature = "odf"))]
    return Err(LinkExtractionError::FeatureNotEnabledError("OpenOffice document detected, but cannot parse it because `odf`-feature is not enabled. Please enable it in your dependencies.".to_string()))
}

fn try_rtf(bytes: &[u8]) -> Result<Vec<String>, LinkExtractionError> {
    #[cfg(feature = "rtf")]
    return Ok(crate::formats::rtf::extract_links(bytes)?);
    #[cfg(not(feature = "rtf"))]
    return Err(LinkExtractionError::FeatureNotEnabledError(".rtf-document detected, but cannot parse it because `rtf`-feature is not enabled. Please enable it in your dependencies.".to_string()))
}

fn try_pdf(bytes: &[u8]) -> Result<Vec<String>, LinkExtractionError> {
    #[cfg(feature = "pdf")]
    return Ok(crate::formats::pdf::extract_links(bytes)?);
    #[cfg(not(feature = "pdf"))]
    return Err(LinkExtractionError::FeatureNotEnabledError("PDF-document detected, but cannot parse it because `pdf`-feature is not enabled. Please enable it in your dependencies.".to_string()))
}

fn try_ooxml(bytes: &[u8]) -> Result<Vec<String>, LinkExtractionError> {
    #[cfg(feature = "ooxml")]
    return Ok(crate::formats::ooxml::extract_links(bytes)?);
    #[cfg(not(feature = "ooxml"))]
    return Err(LinkExtractionError::FeatureNotEnabledError("Microsoft-office document detected, but cannot parse it because `ooxml`-feature is not enabled. Please enable it in your dependencies.".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::include_bytes;

    const TEST_DOCX: &[u8] = include_bytes!("../test_files/docx/test.docx");
    const TEST_PPTX: &[u8] = include_bytes!("../test_files/pptx/test.pptx");
    const TEST_XLSX: &[u8] = include_bytes!("../test_files/xlsx/test.xlsx");
    const TEST_ODT: &[u8] = include_bytes!("../test_files/odt/test.odt");
    const TEST_ODS: &[u8] = include_bytes!("../test_files/ods/test.ods");
    const TEST_OTT: &[u8] = include_bytes!("../test_files/ott/test.ott");
    const TEST_ODP: &[u8] = include_bytes!("../test_files/odp/test.odp");
    const TEST_PDF: &[u8] = include_bytes!("../test_files/pdf/test.pdf");
    const TEST_RTF: &[u8] = include_bytes!("../test_files/rtf/test.rtf");
    const TEST_XML: &[u8] = include_bytes!("../test_files/xml/test.xml");

    #[test]
    fn generic_extraction_test() {
        println!("{:?}", extract_links(b"http://test.com/").unwrap());
        println!("{:?}", extract_links(TEST_DOCX).unwrap());
        println!("{:?}", extract_links(TEST_PPTX).unwrap());
        println!("{:?}", extract_links(TEST_XLSX).unwrap());
        println!("{:?}", extract_links(TEST_ODT).unwrap());
        println!("{:?}", extract_links(TEST_ODS).unwrap());
        println!("{:?}", extract_links(TEST_OTT).unwrap());
        println!("{:?}", extract_links(TEST_ODP).unwrap());
        println!("{:?}", extract_links(TEST_PDF).unwrap());
        println!("{:?}", extract_links(TEST_RTF).unwrap());
        println!("{:?}", extract_links(TEST_XML).unwrap());
    }
}