use std::fmt::{Display, Formatter};
use std::io::read_to_string;
use thiserror::Error;
use crate::link_extractor::find_urls;

#[derive(Error, Debug)]
pub enum LinkExtractionError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[cfg(feature = "xml")]
    #[error(transparent)]
    XmlExtractionError(#[from] crate::formats::xml::XmlExtractionError),

    #[cfg(feature = "svg")]
    #[error(transparent)]
    SvgExtractionError(#[from] crate::formats::xml::svg::SvgExtractionError),

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

#[derive(Debug, Clone)]
pub enum Link {
    StringLink(String),
    #[cfg(feature = "text_file")]
    TextFileLink(crate::formats::text_file::TextFileLink),
    #[cfg(feature = "xml")]
    XmlLink(crate::formats::xml::XmlLink),
    #[cfg(feature = "svg")]
    SvgLink(crate::formats::xml::svg::SvgLink),
    #[cfg(feature = "odf")]
    OdfLink(crate::formats::odf::OdfLink),
    #[cfg(feature = "ooxml")]
    OoxmlLink(crate::formats::ooxml::OoxmlLink),
    #[cfg(feature = "pdf")]
    PdfLink(crate::formats::pdf::PdfLink),
    #[cfg(feature = "rtf")]
    RtfLink(crate::formats::rtf::RtfLink)
}

impl Display for Link {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Link::StringLink(link) => {write!(f, "StringLink({})", link)}
            #[cfg(feature = "text_file")]
            Link::TextFileLink(link) => {write!(f, "TextFileLink({})", link)}
            #[cfg(feature = "xml")]
            Link::XmlLink(link) => {write!(f, "XmlLink({})", link)}
            #[cfg(feature = "svg")]
            Link::SvgLink(link) => {write!(f, "SvgLink({})", link)}
            #[cfg(feature = "odf")]
            Link::OdfLink(link) => {write!(f, "OdfLink({})", link)}
            #[cfg(feature = "ooxml")]
            Link::OoxmlLink(link) => {write!(f, "OoxmlLink({})", link)}
            #[cfg(feature = "pdf")]
            Link::PdfLink(link) => {write!(f, "PdfLink({})", link)}
            #[cfg(feature = "rtf")]
            Link::RtfLink(link) => {write!(f, "RtfLink({})", link)}
        }
    }
}

/// Guesses the file-type and extracts links from the file.
pub fn extract_links(bytes: &[u8]) -> Result<Vec<Link>, LinkExtractionError>{
    let file_type = infer::get(&bytes);

    if file_type == None {
        return Ok(find_urls(&read_to_string(bytes)?).iter().map(|link| Link::StringLink(link.as_str().to_string())).collect());
    }
    let file_type = file_type.unwrap();
    println!("{}", file_type);

    match file_type.mime_type() {
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" => Ok(try_ooxml(bytes)?),
        "application/vnd.oasis.opendocument.text" => Ok(try_odf(bytes)?),
        "application/vnd.oasis.opendocument.spreadsheet" => Ok(try_odf(bytes)?),
        "application/vnd.oasis.opendocument.template" => Ok(try_odf(bytes)?),
        "application/vnd.oasis.opendocument.presentation" => Ok(try_odf(bytes)?),
        "application/zip" => try_zip(bytes),
        "application/pdf" => Ok(try_pdf(bytes)?),
        "application/rtf" => Ok(try_rtf(bytes)?),

        "image/svg+xml" => Ok(try_svg(bytes)?),
        "text/xml" => Ok(try_xml(bytes)?),
        "text/html" => Ok(try_xml(bytes)?),

        "text/plain" => Ok(try_text_file(bytes)?),
        "text/csv" => Ok(try_text_file(bytes)?),
        "text/css" => Ok(try_text_file(bytes)?),
        "application/json" => Ok(try_text_file(bytes)?),
        _ => Err(LinkExtractionError::FileTypeNotImplemented(file_type.mime_type().to_string()))
    }
}

fn try_zip(bytes: &[u8]) -> Result<Vec<Link>, LinkExtractionError> {
    #[allow(unused_assignments)]
        let ret: Result<Vec<Link>, LinkExtractionError> = Err(LinkExtractionError::FeatureNotEnabledError("Zip-file detected. Would try to parse it with `ooxml`/`odf`-feature but none of them is enabled. Please enable it in your dependencies.".to_string()));
    #[cfg(feature = "ooxml")] {
        let ooxml_result = try_ooxml(bytes).map_err(|e| LinkExtractionError::from(e));
        if let Ok(res) = ooxml_result { return Ok(res) }
    }
    #[cfg(feature = "odf")] {
        let odf_result = try_odf(bytes).map_err(|e| LinkExtractionError::from(e));
        if let Ok(res) = odf_result { return Ok(res) }
    }

    return ret;
}


fn try_xml(bytes: &[u8]) -> Result<Vec<Link>, LinkExtractionError> {
    #[cfg(feature = "xml")]
    return Ok(crate::formats::xml::extract_links(bytes)?.into_iter().map(|link| Link::XmlLink(link)).collect());
    #[cfg(not(feature = "xml"))]
    return Err(LinkExtractionError::FeatureNotEnabledError("text-document detected, but cannot parse it because `text_file`-feature is not enabled. Please enable it in your dependencies.".to_string()))
}

fn try_svg(bytes: &[u8]) -> Result<Vec<Link>, LinkExtractionError> {
    #[cfg(feature = "svg")]
    return Ok(crate::formats::xml::svg::extract_links(bytes)?.into_iter().map(|link| Link::SvgLink(link)).collect());
    #[cfg(not(feature = "svg"))]
    return Err(LinkExtractionError::FeatureNotEnabledError("text-document detected, but cannot parse it because `text_file`-feature is not enabled. Please enable it in your dependencies.".to_string()))
}


fn try_text_file(bytes: &[u8]) -> Result<Vec<Link>, LinkExtractionError> {
    #[cfg(feature = "text_file")]
    return Ok(crate::formats::text_file::extract_links(bytes)?.into_iter().map(|link| Link::TextFileLink(link)).collect());
    #[cfg(not(feature = "text_file"))]
    return Err(LinkExtractionError::FeatureNotEnabledError("text-document detected, but cannot parse it because `text_file`-feature is not enabled. Please enable it in your dependencies.".to_string()))
}

fn try_odf(bytes: &[u8]) -> Result<Vec<Link>, LinkExtractionError> {
    #[cfg(feature = "odf")]
    return Ok(crate::formats::odf::extract_links(bytes)?.into_iter().map(|link| Link::OdfLink(link)).collect());
    #[cfg(not(feature = "odf"))]
    return Err(LinkExtractionError::FeatureNotEnabledError("OpenOffice document detected, but cannot parse it because `odf`-feature is not enabled. Please enable it in your dependencies.".to_string()))
}

fn try_rtf(bytes: &[u8]) -> Result<Vec<Link>, LinkExtractionError> {
    #[cfg(feature = "rtf")]
    return Ok(crate::formats::rtf::extract_links(bytes)?.into_iter().map(|link| Link::RtfLink(link)).collect());
    #[cfg(not(feature = "rtf"))]
    return Err(LinkExtractionError::FeatureNotEnabledError(".rtf-document detected, but cannot parse it because `rtf`-feature is not enabled. Please enable it in your dependencies.".to_string()))
}

fn try_pdf(bytes: &[u8]) -> Result<Vec<Link>, LinkExtractionError> {
    #[cfg(feature = "pdf")]
    return Ok(crate::formats::pdf::extract_links(bytes)?.into_iter().map(|link| Link::PdfLink(link)).collect());
    #[cfg(not(feature = "pdf"))]
    return Err(LinkExtractionError::FeatureNotEnabledError("PDF-document detected, but cannot parse it because `pdf`-feature is not enabled. Please enable it in your dependencies.".to_string()))
}

fn try_ooxml(bytes: &[u8]) -> Result<Vec<Link>, LinkExtractionError> {
    #[cfg(feature = "ooxml")]
    return Ok(crate::formats::ooxml::extract_links(bytes)?.into_iter().map(|link| Link::OoxmlLink(link)).collect());
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
    const TEST_SVG: &[u8] = include_bytes!("../test_files/svg/test.svg");

    #[test]
    fn generic_extraction_test() {
        println!("{}", LinkVec(extract_links(b"http://test.com/").unwrap()));
        println!("{}", LinkVec(extract_links(TEST_DOCX).unwrap()));
        println!("{}", LinkVec(extract_links(TEST_PPTX).unwrap()));
        println!("{}", LinkVec(extract_links(TEST_XLSX).unwrap()));
        println!("{}", LinkVec(extract_links(TEST_ODT).unwrap()));
        println!("{}", LinkVec(extract_links(TEST_ODS).unwrap()));
        println!("{}", LinkVec(extract_links(TEST_OTT).unwrap()));
        println!("{}", LinkVec(extract_links(TEST_ODP).unwrap()));
        println!("{}", LinkVec(extract_links(TEST_PDF).unwrap()));
        println!("{}", LinkVec(extract_links(TEST_RTF).unwrap()));
        println!("{}", LinkVec(extract_links(TEST_XML).unwrap()));
        println!("{}", LinkVec(extract_links(TEST_SVG).unwrap()));
    }


    #[derive(Debug)]
    struct LinkVec(Vec<Link>);
    impl Display for LinkVec {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "[")?;
            for link in &self.0 {
                write!(f, "{}, ", link)?;
            }
            write!(f, "]")
        }
    }
}

// TODO sort the files the same way everywhere.