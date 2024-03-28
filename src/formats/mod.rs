#[cfg(feature = "pdf")]
pub mod pdf;
#[cfg(feature = "websites")]
pub mod websites;
#[cfg(feature = "raw_text")]
pub mod raw_text;
#[cfg(feature = "ooxml")]
pub mod ooxml;
#[cfg(feature = "rtf")]
pub mod rtf;
#[cfg(feature = "odt")]
pub mod odt;
#[cfg(any(feature = "odt", feature = "ooxml"))]
mod compressed_formats_common;
