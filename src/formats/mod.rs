#[cfg(feature = "pdf")]
pub mod pdf;
#[cfg(feature = "plaintext")]
/// Any plaintext-format
pub mod plaintext;
#[cfg(feature = "ooxml")]
/// .docx, .pptx, .xlsx
pub mod ooxml;
#[cfg(feature = "rtf")]
pub mod rtf;
#[cfg(feature = "odf")]
/// .odt, .ods, .odp
pub mod odf;
#[cfg(any(feature = "xml", feature = "xlink"))]
/// Also contains xml-based formats
pub mod xml;
#[cfg(any(feature = "odf", feature = "ooxml"))]
mod compressed_formats_common;
#[cfg(feature = "image")]
pub mod image;
