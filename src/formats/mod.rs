#[cfg(feature = "pdf")]
pub mod pdf;
#[cfg(feature = "plaintext")]
pub mod plaintext;
#[cfg(feature = "ooxml")]
pub mod ooxml;
#[cfg(feature = "rtf")]
pub mod rtf;
#[cfg(feature = "odf")]
pub mod odf;
#[cfg(any(feature = "xml", feature = "xlink"))]
pub mod xml;
#[cfg(any(feature = "odf", feature = "ooxml"))]
mod compressed_formats_common;
#[cfg(feature = "image")]
pub mod image;
