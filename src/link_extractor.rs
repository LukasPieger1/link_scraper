use lazy_static::lazy_static;
use linkify::LinkFinder;

lazy_static! {
    static ref FINDER: LinkFinder = LinkFinder::new();
}

//TODO further formats: docx, doc, xlsx, powerpoint, rtf, json, xml, tsv, csv

/// Finds all URLs in a given string
/// # Example
/// ```
/// use crate::untitled_rust_parser::link_extractor::find_links;
/// let urls = find_links("dfjaoijewfj oijoiwfjoiwjoi j´21214https://www.google.com .äwä.f.f.wä ");
/// assert_eq!(urls, vec!["https://www.google.com"])
/// ```
pub fn find_links(content: &str) -> Vec<&str> {
    FINDER.links(content).map(|link| link.as_str()).collect()
}
