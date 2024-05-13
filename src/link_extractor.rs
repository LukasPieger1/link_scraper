use std::hash::Hash;

use itertools::Itertools;
use linkify::LinkFinder;
use linkify::LinkKind::Url;

#[cfg(feature = "generic_file")]
pub use crate::generic_link_extractor::extract_links;

/// Finds all URLs in a given string
/// # Example
/// ```
/// use crate::untitled_rust_parser::link_extractor::find_urls;
/// let urls = find_urls("dfjaoijewfj oijoiwfjoiwjoi j´21214https://www.google.com .äwä.f.f.wä ");
/// assert_eq!(urls, vec!["https://www.google.com"])
/// ```
pub fn find_urls(content: &str) -> Vec<linkify::Link> {
    LinkFinder::new().links(content)
        .filter(|link| link.kind().eq(&Url))
        .collect()
}

pub(crate) fn unique_and_sort<T:Hash + Ord>(arr: &[T]) -> Vec<&T> {
    arr
        .into_iter()
        .sorted()
        .dedup()
        .collect::<Vec<_>>()
}

// TODO wplink /DTD <- main next feature
// TODO for DTD check nom parser
// TODO metadaten für jpg/mp3/...
// TODO more doc
// TODO Readme
// TODO rename project "link_scraper"
// TODO refactor ooxml & odf to use xml-sturcture?