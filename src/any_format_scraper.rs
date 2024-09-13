use crate::gen_scrape_froms;
use crate::helpers::find_urls;
use infer::Type;
use std::fmt::{Display, Formatter};
use std::io::{read_to_string, BufRead, BufReader, Read, Seek};
use thiserror::Error;

/// Guesses the file-type and scrapes links from the file.
pub fn scrape<R>(mut reader: R) -> Result<Vec<Link>, LinkScrapingError>
where
    R: BufRead + Seek,
{
    fn infer_and_scrape<R>(mut reader: R) -> Result<Vec<Link>, LinkScrapingError>
    where
        R: BufRead + Seek,
    {
        if let Some(file_type) = infer::get(reader.fill_buf()?) {
            scrape_from_buffer(reader, file_type)
        } else {
            Ok(find_urls(&read_to_string(reader)?)
                .iter()
                .map(|link| Link::StringLink(link.as_str().to_string()))
                .collect())
        }
    }

    // We do not consume because we want to use the complete buffer later.
    let buf = reader.fill_buf()?;
    match buf.len() {
        0 => Ok(Vec::with_capacity(0)),
        // infer get_from_path uses a buffer of the size 8192 (see infer::Infer::get_from_path)
        // Therefore we haveto make sure,that we grab at least this amount of data when
        // processing it.
        1..8192 => {
            if let Some(found) = infer::get(buf) {
                scrape_from_buffer(reader, found)
            } else {
                infer_and_scrape(BufReader::with_capacity(8192, reader))
            }
        }
        // If we have 8192 or more, we can just use the existing buffer.
        _ => infer_and_scrape(reader),
    }
}
gen_scrape_froms!(scrape(Read) -> Result<Vec<Link>, LinkScrapingError>);

#[derive(Error, Debug)]
pub enum LinkScrapingError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[cfg(feature = "plaintext")]
    #[error(transparent)]
    TextFileScrapingError(#[from] crate::formats::plaintext::TextFileScrapingError),

    #[cfg(feature = "ooxml")]
    #[error(transparent)]
    OoxmlScrapingError(#[from] crate::formats::ooxml::OoxmlScrapingError),

    #[cfg(feature = "odf")]
    #[error(transparent)]
    OdtScrapingError(#[from] crate::formats::odf::OdfScrapingError),

    #[cfg(feature = "pdf")]
    #[error(transparent)]
    PdfScrapingError(#[from] crate::formats::pdf::PdfScrapingError),

    #[cfg(feature = "rtf")]
    #[error(transparent)]
    RtfScrapingError(#[from] crate::formats::rtf::RtfScrapingError),

    #[cfg(feature = "xml")]
    #[error(transparent)]
    XmlScrapingError(#[from] crate::formats::xml::XmlScrapingError),

    #[cfg(feature = "svg")]
    #[error(transparent)]
    SvgScrapingError(#[from] crate::formats::xml::svg::SvgScrapingError),

    #[cfg(feature = "image")]
    #[error(transparent)]
    ImageScrapingError(#[from] crate::formats::image::ImageScrapingError),

    #[error("Required feature is not enabled")]
    FeatureNotEnabledError(String),

    #[error("Filetype not recognized")]
    FileTypeNotImplemented(String),

    #[error("Scraping failed")]
    ScrapingFailedError(String),
}

#[derive(Debug, Clone)]
pub enum Link {
    StringLink(String),
    #[cfg(feature = "plaintext")]
    TextFileLink(crate::formats::plaintext::TextFileLink),
    #[cfg(feature = "odf")]
    OdfLink(crate::formats::odf::OdfLink),
    #[cfg(feature = "pdf")]
    PdfLink(crate::formats::pdf::PdfLink),
    #[cfg(feature = "ooxml")]
    OoxmlLink(crate::formats::ooxml::OoxmlLink),
    #[cfg(feature = "rtf")]
    RtfLink(crate::formats::rtf::RtfLink),
    #[cfg(feature = "xml")]
    XmlLink(crate::formats::xml::XmlLink),
    #[cfg(feature = "svg")]
    SvgLink(crate::formats::xml::svg::SvgLink),
    #[cfg(feature = "image")]
    ImageLink(crate::formats::image::ImageLink),
}

impl Display for Link {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Link::StringLink(link) => {
                write!(f, "StringLink({})", link)
            }
            #[cfg(feature = "plaintext")]
            Link::TextFileLink(link) => {
                write!(f, "TextFileLink({})", link)
            }
            #[cfg(feature = "ooxml")]
            Link::OoxmlLink(link) => {
                write!(f, "OoxmlLink({})", link)
            }
            #[cfg(feature = "odf")]
            Link::OdfLink(link) => {
                write!(f, "OdfLink({})", link)
            }
            #[cfg(feature = "pdf")]
            Link::PdfLink(link) => {
                write!(f, "PdfLink({})", link)
            }
            #[cfg(feature = "rtf")]
            Link::RtfLink(link) => {
                write!(f, "RtfLink({})", link)
            }
            #[cfg(feature = "xml")]
            Link::XmlLink(link) => {
                write!(f, "XmlLink({})", link)
            }
            #[cfg(feature = "svg")]
            Link::SvgLink(link) => {
                write!(f, "SvgLink({})", link)
            }
            #[cfg(feature = "image")]
            Link::ImageLink(link) => {
                write!(f, "ImageLink({})", link)
            }
        }
    }
}

fn scrape_from_buffer<R>(mut reader: R, file_type: Type) -> Result<Vec<Link>, LinkScrapingError>
where
    R: BufRead + Seek,
{
    match file_type.mime_type() {
        "text/plain" | "text/csv" | "text/css" | "application/json" => Ok(try_text_file(reader)?),

        "application/vnd.oasis.opendocument.text"
        | "application/vnd.oasis.opendocument.spreadsheet"
        | "application/vnd.oasis.opendocument.template"
        | "application/vnd.oasis.opendocument.presentation" => Ok(try_odf(reader)?),

        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
        | "application/vnd.openxmlformats-officedocument.spreadsheetml.template"
        | "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
        | "application/vnd.openxmlformats-officedocument.wordprocessingml.template"
        | "application/vnd.openxmlformats-officedocument.presentationml.presentation"
        | "application/vnd.openxmlformats-officedocument.presentationml.template" => Ok(try_ooxml(reader)?),
        | "application/vnd.openxmlformats-officedocument.presentationml.slideshow" => Ok(try_ooxml(reader)?),

        "application/zip" => {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes)?;
            try_zip(bytes)
        }
        "application/pdf" => {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes)?;
            Ok(try_pdf(bytes)?)
        }
        "application/rtf" => Ok(try_rtf(reader)?),
        "image/svg+xml" => Ok(try_svg(reader)?),
        "text/xml" | "text/html" => Ok(try_xml(reader)?),

        "image/jpeg" | "image/png" | "image/tiff" | "image/webp" | "image/heic" | "image/heif" => {
            Ok(try_image(reader)?)
        }

        _ => Err(LinkScrapingError::FileTypeNotImplemented(
            file_type.mime_type().to_string(),
        )),
    }
}

macro_rules! gen_try_format {
    ($name:ident($ty:ty), $feature:literal, $module:ident, $link:ident) => {
        #[cfg(feature = $feature)]
        fn $name(reader: $ty) -> Result<Vec<Link>, LinkScrapingError> {
            return Ok(crate::formats::$module::scrape(reader)?.into_iter().map(|link| Link::$link(link)).collect());
        }

        #[cfg(not(feature = $feature))]
        fn $name(_: $ty) -> Result<Vec<Link>, LinkScrapingError> {
            return Err(LinkScrapingError::FeatureNotEnabledError(format!("Detected {}-file but the corresponding feature is not enabled. Please enable it in your dependencies.", stringify!($feature))));
        }
    }
}

gen_try_format!(
    try_text_file(impl BufRead),
    "plaintext",
    plaintext,
    TextFileLink
);
gen_try_format!(try_ooxml(impl Read + Seek), "ooxml", ooxml, OoxmlLink);
gen_try_format!(try_odf(impl Read + Seek), "odf", odf, OdfLink);
gen_try_format!(try_pdf(impl AsRef<[u8]>), "pdf", pdf, PdfLink);
gen_try_format!(try_rtf(impl Read), "rtf", rtf, RtfLink);
gen_try_format!(try_xml(impl Read), "xml", xml, XmlLink);
gen_try_format!(try_image(impl BufRead + Seek), "image", image, ImageLink);

#[cfg(feature = "svg")]
fn try_svg(reader: impl Read) -> Result<Vec<Link>, LinkScrapingError> {
    return Ok(crate::formats::xml::svg::scrape(reader)?
        .into_iter()
        .map(|link| Link::SvgLink(link))
        .collect());
}
#[cfg(not(feature = "svg"))]
fn try_svg(_: impl Read) -> Result<Vec<Link>, LinkScrapingError> {
    return Err(LinkScrapingError::FeatureNotEnabledError(format!("Detected svg-file but the corresponding feature is not enabled. Please enable it in your dependencies.")));
}

cfg_if::cfg_if! {
    if #[cfg(any(feature = "ooxml", feature = "odf"))] {
        fn try_zip(bytes: impl AsRef<[u8]>) -> Result<Vec<Link>, LinkScrapingError> {
            #[cfg(feature = "ooxml")] {
                let ooxml_result = try_ooxml(std::io::Cursor::new(bytes.as_ref())).map_err(|e| LinkScrapingError::from(e));
                if let Ok(res) = ooxml_result { return Ok(res); }
            }

            #[cfg(feature = "odf")] {
                let odf_result = try_odf(std::io::Cursor::new(bytes.as_ref())).map_err(|e| LinkScrapingError::from(e));
                if let Ok(res) = odf_result { return Ok(res); }
            }

            #[cfg(all(feature = "ooxml", feature = "odf"))] {
                return Err(LinkScrapingError::FileTypeNotImplemented("Detected zip-file but the corresponding type is not supported!".to_string()));
            }
            #[cfg(not(all(feature = "ooxml", feature = "odf")))] {
                return Err(LinkScrapingError::FeatureNotEnabledError("Detected zip-file but the corresponding feature is not enabled. Please enable it in your dependencies.".to_string()));
            }
        }
    } else {
        fn try_zip(_: impl AsRef<[u8]>) -> Result<Vec<Link>, LinkScrapingError> {
            Err(LinkScrapingError::FeatureNotEnabledError("Detected zip-file but the corresponding feature is not enabled. Please enable it in your dependencies.".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;
    use std::include_bytes;

    const TEST_DOCX: &[u8] = include_bytes!("../test_files/ooxml/docx_test.docx");
    const TEST_PPTX: &[u8] = include_bytes!("../test_files/ooxml/pptx_test.pptx");
    const TEST_XLSX: &[u8] = include_bytes!("../test_files/ooxml/xlsx_test.xlsx");
    const TEST_ODT: &[u8] = include_bytes!("../test_files/odf/odt_test.odt");
    const TEST_ODS: &[u8] = include_bytes!("../test_files/odf/ods_test.ods");
    const TEST_ODP: &[u8] = include_bytes!("../test_files/odf/odp_test.odp");
    const TEST_OTT: &[u8] = include_bytes!("../test_files/odf/ott_test.ott");
    const TEST_PDF: &[u8] = include_bytes!("../test_files/pdf/pdf_test.pdf");
    const TEST_RTF: &[u8] = include_bytes!("../test_files/rtf/rtf_test.rtf");
    const TEST_XML: &[u8] = include_bytes!("../test_files/xml/xml_test.xml");
    const TEST_SVG: &[u8] = include_bytes!("../test_files/xml/svg_test.svg");
    const TEST_JPG: &[u8] = include_bytes!("../test_files/images/exif_test.jpg");

    macro_rules! is_active {
        ($name: literal) => {{
            if cfg!(feature = $name) {
                true
            } else {
                false
            }
        }};
    }

    #[test]
    fn scrape_generic_file_test() {
        fn scrape(slice: &[u8], is_active: bool) {
            if is_active {
                println!(
                    "[\"{}\"]",
                    scrape_from_slice(slice)
                        .expect("Failed to scrape a specific format!")
                        .into_iter()
                        .join("\", \"")
                );
            } else {
                scrape_from_slice(slice).expect_err("Expected to fail!");
            }
        }

        scrape(b"https://test.com/", true);
        scrape(TEST_DOCX, is_active!("ooxml"));
        scrape(TEST_PPTX, is_active!("ooxml"));
        scrape(TEST_XLSX, is_active!("ooxml"));
        scrape(TEST_ODT, is_active!("odf"));
        scrape(TEST_ODS, is_active!("odf"));
        scrape(TEST_OTT, is_active!("odf"));
        scrape(TEST_ODP, is_active!("odf"));
        scrape(TEST_PDF, is_active!("pdf"));
        scrape(TEST_RTF, is_active!("rtf"));
        scrape(TEST_XML, is_active!("xml"));
        scrape(TEST_SVG, is_active!("svg"));
        scrape(TEST_JPG, is_active!("image"));
    }
}
