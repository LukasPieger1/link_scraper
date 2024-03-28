#[cfg(feature = "pdf")]
pub mod pdf;
#[cfg(feature = "websites")]
pub mod websites;
#[cfg(feature = "text_file")]
pub mod text_file;
#[cfg(feature = "ooxml")]
pub mod ooxml;
#[cfg(feature = "rtf")]
pub mod rtf;
#[cfg(feature = "odt")]
pub mod odt;
#[cfg(any(feature = "odt", feature = "ooxml"))]
mod compressed_formats_common;
