//! This Crate's main feature are the `scrape`-functions of the different Modules.
//!
//! You can use those functions to extract Links and related information from a file of any supported format.
//!
//! Please refer to the git-projects README.md for known issues and further information.

#[cfg(feature = "any_format")]
/// Use only if you're not sure what format your file will be.
pub mod any_format_scraper;
/// Contains format-specific scrape-functions. Prefer over [`any_format_scraper`].
pub mod formats;
/// Helper functions
pub mod helpers;
